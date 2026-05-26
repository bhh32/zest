//! Equal-cell grid container. Children fill cells in row-major order.
//!
//! ## Scrolling
//!
//! A `Grid` becomes scrollable via [`Grid::scrollable`] plus
//! [`Grid::scroll_state`]. The host owns a [`ScrollState`] (because widgets
//! are transient) and passes it by reference each frame. Unlike the linear
//! containers, a `Grid` supports [`ScrollDirection::Both`] for 2-D panning:
//! the `cols × rows` slot defines the *visible* window and fixes the cell
//! size, while any extra children extend the content beyond the viewport on
//! the scrolling axes. The grid offsets every cell by
//! [`scroll_core::render_offset`], clips the viewport, and draws scrollbars.
//! See [`scroll_core`] for the shared engine. When no
//! scroll is configured the layout/touch/draw paths are identical to a plain
//! `Grid`.

use super::{Widget, element::Element, scroll_core};
use alloc::{boxed::Box, vec::Vec};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{
    Constraints, Length, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState,
    ScrollbarMode, SnapMode, TouchPhase,
};
use zest_theme::Theme;

/// Per-grid scroll configuration (present only when scrolling is enabled).
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

/// Grid of equal-size cells. Children are placed in row-major order.
pub struct Grid<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    cols: u32,
    rows: u32,
    spacing: u32,
    width: Length,
    height: Length,
    scroll: Option<ScrollCore<'a, M>>,
    /// Measured total content size (only meaningful when scrollable).
    content: Size,
    /// Un-scrolled child top-left positions, captured for snap lines.
    child_origins: Vec<Point>,
    /// Un-scrolled child sizes, captured for snap lines.
    child_sizes: Vec<Size>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Grid<'a, C, M> {
    /// Create a `cols x rows` grid. Position and size are assigned by
    /// the parent via `arrange`.
    pub fn new(cols: u32, rows: u32) -> Self {
        Self {
            rect: Rectangle::zero(),
            children: Vec::new(),
            cols,
            rows,
            spacing: 0,
            width: Length::Fill,
            height: Length::Fill,
            scroll: None,
            content: Size::zero(),
            child_origins: Vec::new(),
            child_sizes: Vec::new(),
        }
    }

    /// Gap between cells.
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

    /// Add a child. Excess children beyond `cols × rows` are
    /// retained but not laid out (unless the grid is scrollable, in which
    /// case they extend the content along the scrolling axes).
    #[must_use]
    pub fn push<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.children.push(Element::new(child));
        self
    }

    /// Make this grid scrollable on `dir` (including
    /// [`ScrollDirection::Both`] for 2-D panning). Defaults the scrollbar to
    /// [`ScrollbarMode::Auto`] and no snapping. Pair with
    /// [`Grid::scroll_state`] to supply the host's [`ScrollState`].
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
    /// Implies scrolling (defaults to [`ScrollDirection::Both`] if
    /// [`Grid::scrollable`] was not called first).
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        let core = self.scroll.get_or_insert(ScrollCore {
            state: *state,
            dir: ScrollDirection::Both,
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
            dir: ScrollDirection::Both,
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
            dir: ScrollDirection::Both,
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
            dir: ScrollDirection::Both,
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
        if self.cols == 0 || self.rows == 0 {
            return;
        }

        let h_spacing = self.spacing * (self.cols.saturating_sub(1));
        let v_spacing = self.spacing * (self.rows.saturating_sub(1));
        let cell_width = self.rect.size.width.saturating_sub(h_spacing) / self.cols;
        let cell_height = self.rect.size.height.saturating_sub(v_spacing) / self.rows;

        for (idx, child) in self.children.iter_mut().enumerate() {
            let row = (idx as u32) / self.cols;
            let col = (idx as u32) % self.cols;

            if row >= self.rows {
                break;
            }

            let x = self.rect.top_left.x + (col * (cell_width + self.spacing)) as i32;
            let y = self.rect.top_left.y + (row * (cell_height + self.spacing)) as i32;

            child.arrange(Rectangle::new(
                Point::new(x, y),
                Size::new(cell_width, cell_height),
            ));
        }
    }

    // ---- scrolling layout ---------------------------------------------

    fn relayout_scroll(&mut self, dir: ScrollDirection) {
        self.child_origins.clear();
        self.child_sizes.clear();
        if self.cols == 0 || self.rows == 0 {
            self.content = Size::zero();
            return;
        }

        // The `cols × rows` slot fixes the visible cell size. Extra children
        // extend the content beyond the viewport on the scrolling axes.
        let h_spacing = self.spacing * (self.cols.saturating_sub(1));
        let v_spacing = self.spacing * (self.rows.saturating_sub(1));
        let cell_width = self.rect.size.width.saturating_sub(h_spacing) / self.cols.max(1);
        let cell_height = self.rect.size.height.saturating_sub(v_spacing) / self.rows.max(1);

        let n = self.children.len() as u32;
        let total_rows = if n == 0 { 0 } else { n.div_ceil(self.cols) };

        // Content extent: full grid of all children when scrolling that axis,
        // otherwise the viewport extent.
        let content_w = if dir.scrolls_x() {
            (cell_width.saturating_add(self.spacing))
                .saturating_mul(self.cols)
                .saturating_sub(self.spacing)
        } else {
            self.rect.size.width
        };
        let content_h = if dir.scrolls_y() {
            (cell_height.saturating_add(self.spacing))
                .saturating_mul(total_rows)
                .saturating_sub(self.spacing)
        } else {
            self.rect.size.height
        };
        self.content = Size::new(content_w, content_h.max(self.rect.size.height));

        let off = self
            .scroll
            .as_ref()
            .map_or(Point::zero(), |c| scroll_core::render_offset(c.state, dir));
        let base_x = self.rect.top_left.x - off.x;
        let base_y = self.rect.top_left.y - off.y;

        for (idx, child) in self.children.iter_mut().enumerate() {
            let row = (idx as u32) / self.cols;
            let col = (idx as u32) % self.cols;

            let x = base_x + (col * (cell_width + self.spacing)) as i32;
            let y = base_y + (row * (cell_height + self.spacing)) as i32;
            let origin = Point::new(x, y);
            let size = Size::new(cell_width, cell_height);
            child.arrange(Rectangle::new(origin, size));
            self.child_origins.push(origin);
            self.child_sizes.push(size);
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Grid<'a, C, M> {
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
                let content = self.content;
                let lines = self.snap_lines();
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
                for child in &self.children {
                    child.draw(renderer, theme)?;
                }
                Ok(())
            }
        }
    }
}
