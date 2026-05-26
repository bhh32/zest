//! Vertical layout container. Runs a flex resolve modeled on iced's
//! `core/src/layout/flex.rs`:
//!
//! 1. allocate `Length::Fixed` slots,
//! 2. measure `Length::Shrink` children with the residual constraint,
//! 3. divide what's left across `Length::Fill` / `Length::FillPortion`
//!    children proportional to their portion weights.
//!
//! ## Scrolling
//!
//! A `Column` becomes scrollable via [`Column::scrollable`] plus
//! [`Column::scroll_state`]. The host owns a [`ScrollState`] (because widgets
//! are transient) and passes it by reference each frame; the column reads it
//! during layout/draw and emits [`ScrollMsg`] through [`Column::on_scroll`].
//! See [`scroll_core`](super::scroll_core) for the shared engine. When no
//! scroll is configured the layout/touch/draw paths are identical to a plain
//! `Column`.

use super::{Widget, element::Element, scroll_core};
use alloc::{boxed::Box, vec::Vec};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{
    Constraints, Length, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState,
    ScrollbarMode, SnapMode, TouchPhase, UiAction, WidgetId,
};
use zest_theme::Theme;

/// Per-column scroll configuration (present only when scrolling is enabled).
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

/// Vertical stack of widgets.
pub struct Column<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    spacing: u32,
    width: Length,
    height: Length,
    scroll: Option<ScrollCore<'a, M>>,
    /// Measured total content height (only meaningful when scrollable).
    content_h: u32,
    /// Un-scrolled child top-left positions, captured for snap lines.
    child_origins: Vec<Point>,
    /// Un-scrolled child sizes, captured for snap lines.
    child_sizes: Vec<Size>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Column<'a, C, M> {
    /// Create a new empty column. Position and size are assigned by
    /// the parent via `arrange`.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            children: Vec::new(),
            spacing: 2,
            width: Length::Fill,
            height: Length::Fill,
            scroll: None,
            content_h: 0,
            child_origins: Vec::new(),
            child_sizes: Vec::new(),
        }
    }

    /// Gap between children.
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
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

    /// Add a child.
    #[must_use]
    pub fn push<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.children.push(Element::new(child));
        self
    }

    /// Make this column scrollable on `dir`. Defaults the scrollbar
    /// to [`ScrollbarMode::Auto`] and no snapping. Pair with
    /// [`Column::scroll_state`] to supply the host's [`ScrollState`].
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
    /// [`Column::scrollable`] was not called first).
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

    /// Snap-line candidates for the current layout, in offset space. Empty
    /// when not snapping.
    fn snap_lines(&self) -> Vec<i32> {
        match &self.scroll {
            Some(core) if core.snap != SnapMode::None => {
                let rects: Vec<Rectangle> = self
                    .child_origins
                    .iter()
                    .zip(self.child_sizes.iter())
                    .map(|(p, s)| Rectangle::new(*p, *s))
                    .collect();
                let offset = scroll_core::render_offset(core.state, core.dir);
                scroll_core::snap_lines(
                    &rects,
                    self.rect.top_left,
                    offset,
                    self.rect.size,
                    core.dir,
                    core.snap,
                )
            }
            _ => Vec::new(),
        }
    }

    // ---- non-scrolling layout ------

    fn relayout(&mut self) {
        let n = self.children.len();
        if n == 0 {
            return;
        }
        let avail_w = self.rect.size.width;
        let total_spacing = self.spacing.saturating_mul(n as u32 - 1);
        let avail_h = self.rect.size.height.saturating_sub(total_spacing);

        // Pass 1 + 2: collect per-child intent + height.
        let cross = Constraints::loose(Size::new(avail_w, avail_h));
        let mut heights: Vec<u32> = Vec::with_capacity(n);
        let mut fixed_total: u32 = 0;
        let mut shrink_total: u32 = 0;
        let mut portion_total: u32 = 0;

        for child in &mut self.children {
            let (_, h_intent) = child.preferred_size();
            let h = match h_intent {
                Length::Fixed(px) => {
                    fixed_total = fixed_total.saturating_add(px);
                    px
                }
                Length::Shrink => {
                    let m = child.measure(cross).height;
                    shrink_total = shrink_total.saturating_add(m);
                    m
                }
                Length::Fill | Length::FillPortion(_) => {
                    portion_total = portion_total.saturating_add(h_intent.portion());
                    0
                }
            };
            heights.push(h);
        }

        // Pass 3: distribute remaining height to Fill / FillPortion.
        let consumed = fixed_total.saturating_add(shrink_total);
        let remaining = avail_h.saturating_sub(consumed);
        if portion_total > 0 {
            let unit = remaining / portion_total;
            for (child, h) in self.children.iter_mut().zip(heights.iter_mut()) {
                let (_, h_intent) = child.preferred_size();
                if h_intent == Length::Fill || matches!(h_intent, Length::FillPortion(_)) {
                    *h = unit.saturating_mul(h_intent.portion());
                }
            }
        }

        // Pass 4: arrange.
        let mut y = self.rect.top_left.y;
        for (child, h) in self.children.iter_mut().zip(heights.iter()) {
            let cell = Rectangle::new(Point::new(self.rect.top_left.x, y), Size::new(avail_w, *h));
            child.arrange(cell);
            y += *h as i32 + self.spacing as i32;
        }
    }

    // ---- scrolling layout ---------------------------------------------

    fn relayout_scroll(&mut self, dir: ScrollDirection) {
        let n = self.children.len();
        self.child_origins.clear();
        self.child_sizes.clear();
        if n == 0 {
            self.content_h = 0;
            return;
        }
        let scrolls_y = dir.scrolls_y();
        // The scrollbar overlays the content edge (drawn after pop_clip),
        // LVGL-style, so children use the full viewport width.
        let avail_w = self.rect.size.width;
        let spacing = self.spacing;

        // On the scroll axis, measure each child against UNBOUNDED so the
        // column learns its intrinsic content extent (mirrors scrollable.rs).
        let measure_h = if scrolls_y {
            zest_core::UNBOUNDED
        } else {
            self.rect
                .size
                .height
                .saturating_sub(spacing.saturating_mul(n as u32 - 1))
        };
        let cross = Constraints::loose(Size::new(avail_w, measure_h));

        let mut heights: Vec<u32> = Vec::with_capacity(n);
        for child in &mut self.children {
            let (_, h_intent) = child.preferred_size();
            let h = match h_intent {
                Length::Fixed(px) => px,
                // `Fill` is meaningless on an unbounded scroll axis — measuring
                // it would resolve to `UNBOUNDED` and explode the content
                // height (a single child taller than the screen, leaving the
                // rest unreachable). Clamp such a child to one viewport.
                Length::Fill | Length::FillPortion(_) if scrolls_y => self.rect.size.height,
                _ => child.measure(cross).height,
            };
            heights.push(h);
        }

        let total_spacing = spacing.saturating_mul(n as u32 - 1);
        let content_h: u32 = heights
            .iter()
            .copied()
            .sum::<u32>()
            .saturating_add(total_spacing);
        self.content_h = content_h;

        // Resolve the render offset from the host-owned state.
        let off = self
            .scroll
            .as_ref()
            .map_or(Point::zero(), |c| scroll_core::render_offset(c.state, dir));

        let mut y = self.rect.top_left.y - off.y;
        let x = self.rect.top_left.x - off.x;
        for (child, h) in self.children.iter_mut().zip(heights.iter()) {
            let origin = Point::new(x, y);
            let size = Size::new(avail_w, *h);
            child.arrange(Rectangle::new(origin, size));
            self.child_origins.push(origin);
            self.child_sizes.push(size);
            y += *h as i32 + spacing as i32;
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Column<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Column<'a, C, M> {
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
        match self.scroll.as_ref().map(|c| c.dir) {
            Some(dir) if dir != ScrollDirection::None => self.relayout_scroll(dir),
            _ => self.relayout(),
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
                let viewport = self.rect;
                let content = Size::new(self.rect.size.width, self.content_h);
                let lines = self.snap_lines();
                // Take the callback out so the closure can borrow children
                // mutably without aliasing `self.scroll`.
                let on_scroll = self.scroll.as_ref().and_then(|c| c.on_scroll.as_deref());
                let children = &mut self.children;
                scroll_core::route_touch(
                    state,
                    dir,
                    viewport,
                    content,
                    point,
                    phase,
                    &lines,
                    on_scroll,
                    |p, ph| {
                        for child in children.iter_mut().rev() {
                            if let Some(msg) = child.handle_touch(p, ph) {
                                return Some(msg);
                            }
                        }
                        None
                    },
                )
            }
            _ => {
                for child in self.children.iter_mut().rev() {
                    if let Some(msg) = child.handle_touch(point, phase) {
                        return Some(msg);
                    }
                }
                None
            }
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
        for child in &mut self.children {
            child.mark_pressed(point);
        }
    }

    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        for child in &self.children {
            child.collect_focusable(out);
        }
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        for child in &mut self.children {
            child.sync_focus(focused);
        }
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        for child in &mut self.children {
            if let Some(msg) = child.route_action(target, action) {
                return Some(msg);
            }
        }
        None
    }

    fn navigate_focus(&self, target: WidgetId, action: UiAction) -> Option<WidgetId> {
        for child in &self.children {
            if let Some(next) = child.navigate_focus(target, action) {
                return Some(next);
            }
        }
        None
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        for child in &self.children {
            if let Some(rect) = child.focus_rect(target) {
                return Some(rect);
            }
        }
        None
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        for child in self.children.iter().rev() {
            if let Some(id) = child.focus_at(point) {
                return Some(id);
            }
        }
        None
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        match self.scroll.as_ref() {
            Some(core) if core.dir != ScrollDirection::None => {
                let viewport = self.rect;
                renderer.push_clip(viewport);
                for child in &self.children {
                    child.draw(renderer, theme)?;
                }
                renderer.pop_clip();
                let content = Size::new(self.rect.size.width, self.content_h);
                scroll_core::draw_scrollbars(
                    renderer, theme, core.state, core.bar, core.dir, viewport, content,
                )?;
                Ok(())
            }
            _ => {
                for child in &self.children {
                    child.draw(renderer, theme)?;
                }
                Ok(())
            }
        }
    }
}
