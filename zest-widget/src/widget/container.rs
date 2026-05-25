//! Bounds-aware single-child wrapper. Applies padding and forwards
//! the touch / draw protocol to its inner widget.
//!
//! ## Scrolling
//!
//! A `Container` becomes scrollable via [`Container::scrollable`] plus
//! [`Container::scroll_state`]. The host owns a [`ScrollState`] (because
//! widgets are transient) and passes it by reference each frame; the
//! container reads it during layout/draw and emits [`ScrollMsg`] through
//! [`Container::on_scroll`]. The single child is measured against
//! `UNBOUNDED` on the scrolling axis so its intrinsic content extent is
//! known, then offset by [`scroll_core::render_offset`]. See
//! [`scroll_core`](super::scroll_core) for the shared engine. When no
//! scroll is configured the layout/touch/draw paths are identical to a
//! plain `Container`.

use super::{Widget, element::Element, scroll_core};
use alloc::boxed::Box;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{
    Constraints, Length, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState,
    ScrollbarMode, SnapMode, TouchPhase,
};
use zest_theme::Theme;

/// Per-container scroll configuration (present only when scrolling is enabled).
struct ScrollCore<'a, M> {
    /// Host-owned scroll state, read each frame.
    state: ScrollState,
    /// Which axes scroll.
    dir: ScrollDirection,
    /// When the scrollbar is shown.
    bar: ScrollbarMode,
    /// How scrolling settles to child boundaries.
    snap: SnapMode,
    /// Callback turning a [`ScrollMsg`] into the host message.
    on_scroll: Option<Box<dyn Fn(ScrollMsg) -> M + 'a>>,
}

/// Holds one child inside a bounded region with optional padding.
pub struct Container<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    padding: u32,
    child: Option<Element<'a, C, M>>,
    width: Length,
    height: Length,
    scroll: Option<ScrollCore<'a, M>>,
    /// Measured content size of the child (only meaningful when scrollable).
    content: Size,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Container<'a, C, M> {
    /// Create a new empty container. The parent assigns position and
    /// size via `arrange`; use `.width(...)` / `.height(...)` to
    /// constrain the slot.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            padding: 0,
            child: None,
            width: Length::Fill,
            height: Length::Fill,
            scroll: None,
            content: Size::zero(),
        }
    }

    /// Padding inset on all sides.
    #[must_use]
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Set the child widget. The parent will call `arrange`
    /// later, which propagates the inner rect to the child.
    #[must_use]
    pub fn child<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.child = Some(Element::new(child));
        self
    }

    /// Make this container scrollable on `dir`. Defaults the
    /// scrollbar to [`ScrollbarMode::Auto`] and no snapping. Pair with
    /// [`Container::scroll_state`] to supply the host's [`ScrollState`].
    #[must_use]
    pub fn scrollable(mut self, dir: ScrollDirection) -> Self {
        let core = self.scroll.get_or_insert(ScrollCore {
            state: ScrollState::new(),
            dir,
            bar: ScrollbarMode::Auto,
            snap: SnapMode::None,
            on_scroll: None,
        });
        core.dir = dir;
        self
    }

    /// Supply the host-owned [`ScrollState`] read this frame.
    /// Implies scrolling (defaults to [`ScrollDirection::Vertical`] if
    /// [`Container::scrollable`] was not called first).
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        let core = self.scroll.get_or_insert(ScrollCore {
            state: *state,
            dir: ScrollDirection::Vertical,
            bar: ScrollbarMode::Auto,
            snap: SnapMode::None,
            on_scroll: None,
        });
        core.state = *state;
        self
    }

    /// When the scrollbar is drawn. Implies scrolling.
    #[must_use]
    pub fn scrollbar(mut self, mode: ScrollbarMode) -> Self {
        let core = self.scroll.get_or_insert(ScrollCore {
            state: ScrollState::new(),
            dir: ScrollDirection::Vertical,
            bar: mode,
            snap: SnapMode::None,
            on_scroll: None,
        });
        core.bar = mode;
        self
    }

    /// Snapping mode. Implies scrolling.
    #[must_use]
    pub fn snap(mut self, mode: SnapMode) -> Self {
        let core = self.scroll.get_or_insert(ScrollCore {
            state: ScrollState::new(),
            dir: ScrollDirection::Vertical,
            bar: ScrollbarMode::Auto,
            snap: mode,
            on_scroll: None,
        });
        core.snap = mode;
        self
    }

    /// Callback mapping a [`ScrollMsg`] to the host message. Implies
    /// scrolling.
    #[must_use]
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(ScrollMsg) -> M + 'a,
    {
        let core = self.scroll.get_or_insert(ScrollCore {
            state: ScrollState::new(),
            dir: ScrollDirection::Vertical,
            bar: ScrollbarMode::Auto,
            snap: SnapMode::None,
            on_scroll: None,
        });
        core.on_scroll = Some(Box::new(f));
        self
    }

    fn inner_rect(&self) -> Rectangle {
        let pad = self.padding as i32;
        let pad_u = self.padding;
        Rectangle::new(
            self.rect.top_left + Point::new(pad, pad),
            Size::new(
                self.rect.size.width.saturating_sub(pad_u * 2),
                self.rect.size.height.saturating_sub(pad_u * 2),
            ),
        )
    }

    /// Snap-line candidates for the current layout, in offset space. Empty
    /// when not snapping.
    fn snap_lines(&self) -> alloc::vec::Vec<i32> {
        match &self.scroll {
            Some(core) if core.snap != SnapMode::None => {
                let viewport = self.inner_rect();
                let off = scroll_core::render_offset(core.state, core.dir);
                // Reconstruct the un-scrolled child rect from cached content.
                let rect = Rectangle::new(viewport.top_left, self.content);
                scroll_core::snap_lines(
                    &[rect],
                    viewport.top_left,
                    off,
                    viewport.size,
                    core.dir,
                    core.snap,
                )
            }
            _ => alloc::vec::Vec::new(),
        }
    }

    /// Layout the child for the scrolling path: measure against UNBOUNDED on
    /// the scrolling axis, record `content`, and offset by `-render_offset`.
    fn arrange_scroll(&mut self, dir: ScrollDirection) {
        let viewport = self.inner_rect();
        let measure_w = if dir.scrolls_x() {
            zest_core::UNBOUNDED
        } else {
            viewport.size.width
        };
        let measure_h = if dir.scrolls_y() {
            zest_core::UNBOUNDED
        } else {
            viewport.size.height
        };
        let off = self
            .scroll
            .as_ref()
            .map_or(Point::zero(), |c| scroll_core::render_offset(c.state, dir));
        if let Some(child) = self.child.as_mut() {
            let measured = child.measure(Constraints::loose(Size::new(measure_w, measure_h)));
            // The child fills the viewport on a non-scrolling axis and uses
            // its intrinsic size on a scrolling one.
            let content = Size::new(
                if dir.scrolls_x() {
                    measured.width.max(viewport.size.width)
                } else {
                    viewport.size.width
                },
                if dir.scrolls_y() {
                    measured.height.max(viewport.size.height)
                } else {
                    viewport.size.height
                },
            );
            self.content = content;
            child.arrange(Rectangle::new(viewport.top_left - off, content));
        } else {
            self.content = Size::zero();
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Container<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Container<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let dx = 2 * self.padding;
        let dy = 2 * self.padding;
        let inner = constraints.shrink(dx, dy);
        let child_size = self
            .child
            .as_mut()
            .map_or(Size::zero(), |c| c.measure(inner));
        let intrinsic_w = child_size.width.saturating_add(dx);
        let intrinsic_h = child_size.height.saturating_add(dy);
        let w = self.width.resolve(intrinsic_w, constraints.max.width);
        let h = self.height.resolve(intrinsic_h, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
        match self.scroll.as_ref().map(|c| c.dir) {
            Some(dir) if dir != ScrollDirection::None => self.arrange_scroll(dir),
            _ => {
                let inner = self.inner_rect();
                if let Some(child) = self.child.as_mut() {
                    child.arrange(inner);
                }
            }
        }
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        match self.scroll.as_ref() {
            Some(core) if core.dir != ScrollDirection::None => {
                let dir = core.dir;
                let state = core.state;
                let viewport = self.inner_rect();
                let content = self.content;
                let lines = self.snap_lines();
                let on_scroll = self.scroll.as_ref().and_then(|c| c.on_scroll.as_deref());
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
                    |p, ph| child.as_mut().and_then(|c| c.handle_touch(p, ph)),
                )
            }
            _ => self
                .child
                .as_mut()
                .and_then(|child| child.handle_touch(point, phase)),
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        // While dragging/flinging/springing, stop re-asserting a child press
        // so any button highlighted on Down is cancelled mid-drag.
        if let Some(core) = self.scroll.as_ref() {
            if matches!(
                core.state.phase,
                zest_core::GesturePhase::Dragging
                    | zest_core::GesturePhase::Flinging
                    | zest_core::GesturePhase::Springing
            ) {
                return;
            }
        }
        if let Some(child) = self.child.as_mut() {
            child.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        match self.scroll.as_ref() {
            Some(core) if core.dir != ScrollDirection::None => {
                let viewport = self.inner_rect();
                renderer.push_clip(viewport);
                if let Some(child) = &self.child {
                    child.draw(renderer, theme)?;
                }
                renderer.pop_clip();
                scroll_core::draw_scrollbars(
                    renderer,
                    theme,
                    core.state,
                    core.bar,
                    core.dir,
                    viewport,
                    self.content,
                )?;
                Ok(())
            }
            _ => {
                if let Some(child) = &self.child {
                    child.draw(renderer, theme)?;
                }
                Ok(())
            }
        }
    }
}
