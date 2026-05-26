//! Drag-to-scroll single-child wrapper (the iced `Scrollable` analog).
//!
//! Wraps one child and makes it scrollable with the shared scroll engine: a
//! 1:1 drag pans the content, a fast release flings with friction,
//! over-dragging an edge stretches then springs back, and a thumb shows the
//! position.
//!
//! Like the scrollable containers, the host owns a [`ScrollState`] (widgets
//! are transient) and passes it via [`Scrollable::scroll_state`]; the wrapper
//! reads it during layout/draw and emits [`ScrollMsg`] through
//! [`Scrollable::on_scroll`]. The host applies those in `update()` and drives
//! momentum with [`tick_task`](zest_core::scroll::tick_task). For multi-child
//! content, make a `Column`/`Row`/`Grid` scrollable directly instead.

use super::{Widget, element::Element, scroll_core};
use alloc::{boxed::Box, vec::Vec};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{
    Constraints, GesturePhase, Length, RenderError, Renderer, ScrollDirection, ScrollMsg,
    ScrollState, ScrollbarMode, SnapMode, TouchPhase, UNBOUNDED, UiAction, WidgetId,
};
use zest_theme::Theme;

/// A scrollable viewport wrapping a single child.
pub struct Scrollable<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    child: Element<'a, C, M>,
    dir: ScrollDirection,
    bar: ScrollbarMode,
    snap: SnapMode,
    state: ScrollState,
    on_scroll: Option<Box<dyn Fn(ScrollMsg) -> M + 'a>>,
    width: Length,
    height: Length,
    /// Scrollable content extent measured this frame.
    content: Size,
    /// Un-scrolled child origin + size (for snap-line computation).
    child_origin: Point,
    child_size: Size,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Scrollable<'a, C, M> {
    /// Wrap `child` in a vertically scrollable viewport. Position and size are
    /// assigned by the parent via `arrange`.
    pub fn new<W>(child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        Self {
            rect: Rectangle::zero(),
            child: Element::new(child),
            dir: ScrollDirection::Vertical,
            bar: ScrollbarMode::Auto,
            snap: SnapMode::None,
            state: ScrollState::new(),
            on_scroll: None,
            width: Length::Fill,
            height: Length::Fill,
            content: Size::zero(),
            child_origin: Point::zero(),
            child_size: Size::zero(),
        }
    }

    /// Which axes scroll (default [`ScrollDirection::Vertical`]).
    #[must_use]
    pub fn direction(mut self, dir: ScrollDirection) -> Self {
        self.dir = dir;
        self
    }

    /// Supply the host-owned [`ScrollState`] read this frame.
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        self.state = *state;
        self
    }

    /// When the scrollbar is drawn (default [`ScrollbarMode::Auto`]).
    #[must_use]
    pub fn scrollbar(mut self, mode: ScrollbarMode) -> Self {
        self.bar = mode;
        self
    }

    /// Snapping mode (default [`SnapMode::None`]).
    #[must_use]
    pub fn snap(mut self, mode: SnapMode) -> Self {
        self.snap = mode;
        self
    }

    /// Callback mapping a [`ScrollMsg`] to the host message. Without
    /// it, scroll offsets are still computed but never reach the host, so the
    /// position isn't persisted across frames.
    #[must_use]
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(ScrollMsg) -> M + 'a,
    {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    fn snap_lines(&self) -> Vec<i32> {
        if self.snap == SnapMode::None {
            return Vec::new();
        }
        let offset = scroll_core::render_offset(self.state, self.dir);
        scroll_core::snap_lines(
            &[Rectangle::new(self.child_origin, self.child_size)],
            self.rect.top_left,
            offset,
            self.rect.size,
            self.dir,
            self.snap,
        )
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Scrollable<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self
            .height
            .resolve(constraints.max.height, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
        let dir = self.dir;
        let (vw, vh) = (rect.size.width, rect.size.height);
        // Measure the child against UNBOUNDED on the scrolling axis to learn
        // its intrinsic content extent.
        let max_w = if dir.scrolls_x() { UNBOUNDED } else { vw };
        let max_h = if dir.scrolls_y() { UNBOUNDED } else { vh };
        let m = self
            .child
            .measure(Constraints::loose(Size::new(max_w, max_h)));
        self.content = Size::new(
            if dir.scrolls_x() { m.width } else { vw },
            if dir.scrolls_y() { m.height } else { vh },
        );
        let off = scroll_core::render_offset(self.state, dir);
        let size = Size::new(self.content.width.max(vw), self.content.height.max(vh));
        self.child
            .arrange(Rectangle::new(rect.top_left - off, size));
        self.child_origin = rect.top_left;
        self.child_size = size;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        let state = self.state;
        let dir = self.dir;
        let viewport = self.rect;
        let content = self.content;
        let lines = self.snap_lines();
        let on_scroll = self.on_scroll.as_deref();
        let child = &mut self.child;
        scroll_core::route_touch(
            state,
            dir,
            viewport,
            content,
            point,
            phase,
            &lines,
            on_scroll,
            |p, ph| child.handle_touch(p, ph),
        )
    }

    fn mark_pressed(&mut self, point: Point) {
        // Cancel any child press once a drag/animation owns the gesture.
        if matches!(
            self.state.phase,
            GesturePhase::Dragging | GesturePhase::Flinging | GesturePhase::Springing
        ) {
            return;
        }
        self.child.mark_pressed(point);
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        renderer.push_clip(self.rect);
        self.child.draw(renderer, theme)?;
        renderer.pop_clip();
        scroll_core::draw_scrollbars(
            renderer,
            theme,
            self.state,
            self.bar,
            self.dir,
            self.rect,
            self.content,
        )
    }

    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        self.child.collect_focusable(out);
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        self.child.sync_focus(focused);
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        self.child.route_action(target, action)
    }

    fn navigate_focus(&self, target: WidgetId, action: UiAction) -> Option<WidgetId> {
        self.child.navigate_focus(target, action)
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        self.child.focus_rect(target)
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        self.child.focus_at(point)
    }
}
