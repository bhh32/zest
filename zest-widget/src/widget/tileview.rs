//! Full-area paged tiles that settle exactly one tile per swipe.
//!
//! A `Tileview` lays its children out edge-to-edge along one axis — each tile
//! filling the whole viewport — and uses the shared scroll engine with
//! [`zest_core::SnapMode::Start`] so every release settles to a tile boundary. Combined
//! with one snap line per tile, a flick advances or retreats exactly one tile
//! (the spring always picks the nearest boundary).
//!
//! Scroll direction is chosen with [`Tileview::direction`]: horizontal pages
//! left/right, vertical pages up/down. (Use [`ScrollDirection::Horizontal`] or
//! [`ScrollDirection::Vertical`]; other values fall back to horizontal.)
//!
//! ## Host ownership
//!
//! As with every scrollable widget in `zest`, cross-frame state lives on the
//! host (widgets are transient — see [`scroll`](zest_core::scroll)). The host
//! owns a [`ScrollState`] plus the current tile index and passes the state by
//! reference each frame via [`Tileview::scroll_state`]. The tileview reads it
//! during layout/draw and emits:
//!
//! - [`ScrollMsg`] through [`Tileview::on_scroll`] — apply to the owned
//!   [`ScrollState`] in `update()` exactly as for a scrollable
//!   [`Column`](super::column::Column).
//! - the active tile index through [`Tileview::on_change`] — the host computes
//!   it from its owned state with [`Tileview::current_for`] after each
//!   [`ScrollMsg`]/`tick`, so the index follows the swipe and commits when the
//!   page settles.

use super::{Widget, element::Element, scroll_core};
use alloc::{boxed::Box, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{
    Constraints, Length, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState, TouchPhase,
};
use zest_theme::Theme;

/// Full-area paged tile container. Snaps one tile per swipe.
pub struct Tileview<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    /// Tiles, in order; each fills the whole viewport.
    tiles: Vec<Element<'a, C, M>>,
    /// Host-owned scroll state, read each frame.
    state: ScrollState,
    /// Paging axis (Horizontal or Vertical).
    dir: ScrollDirection,
    /// Callback turning a [`ScrollMsg`] into the host message.
    on_scroll: Option<Box<dyn Fn(ScrollMsg) -> M + 'a>>,
    /// Callback firing the active tile index when it changes (host-driven).
    on_change: Option<Box<dyn Fn(usize) -> M + 'a>>,
    width: Length,
    height: Length,
    /// Cached content extent along the paging axis (tiles laid edge-to-edge).
    content: Size,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Tileview<'a, C, M> {
    /// Create an empty tileview paging horizontally. Add tiles via
    /// [`Tileview::push`] and supply the host state via
    /// [`Tileview::scroll_state`]. Position and size are assigned by the
    /// parent container via `arrange`.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            tiles: Vec::new(),
            state: ScrollState::new(),
            dir: ScrollDirection::Horizontal,
            on_scroll: None,
            on_change: None,
            width: Length::Fill,
            height: Length::Fill,
            content: Size::zero(),
            _color: PhantomData,
        }
    }

    /// Paging direction. Only [`ScrollDirection::Horizontal`] and
    /// [`ScrollDirection::Vertical`] are meaningful; any other value is
    /// treated as horizontal.
    #[must_use]
    pub fn direction(mut self, dir: ScrollDirection) -> Self {
        self.dir = match dir {
            ScrollDirection::Vertical => ScrollDirection::Vertical,
            _ => ScrollDirection::Horizontal,
        };
        self
    }

    /// Push a tile. Tiles are paged in insertion order.
    #[must_use]
    pub fn push<W>(mut self, tile: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.tiles.push(Element::new(tile));
        self
    }

    /// Supply the host-owned [`ScrollState`] read this frame.
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        self.state = *state;
        self
    }

    /// Width sizing intent (the tileview normally fills its parent).
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent (the tileview normally fills its parent).
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Callback mapping a [`ScrollMsg`] to the host message. Apply the
    /// message to the owned [`ScrollState`] in `update()`.
    #[must_use]
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(ScrollMsg) -> M + 'a,
    {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Callback fired with the active tile index when it changes.
    /// Drive it from `update()` via [`Tileview::change_msg`] /
    /// [`Tileview::current_for`].
    #[must_use]
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> M + 'a,
    {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Snap-line offsets — one per tile boundary along the paging axis — so a
    /// release always settles exactly one tile into the viewport.
    fn snap_lines(&self) -> Vec<i32> {
        let extent = if self.dir == ScrollDirection::Vertical {
            self.rect.size.height as i32
        } else {
            self.rect.size.width as i32
        };
        (0..self.tiles.len() as i32).map(|i| i * extent).collect()
    }

    /// Active tile index for the current rendered offset (nearest boundary).
    fn current_index(&self) -> usize {
        Self::current_for(&self.state, self.rect.size, self.dir, self.tiles.len())
    }

    /// The active tile index for a host [`ScrollState`], the `viewport` size,
    /// the paging `dir`, and tile `count`. The host calls this in `update()`
    /// after applying a [`ScrollMsg`]/`tick` to learn which tile is showing,
    /// without rebuilding the widget tree.
    #[must_use]
    pub fn current_for(
        state: &ScrollState,
        viewport: Size,
        dir: ScrollDirection,
        count: usize,
    ) -> usize {
        if count == 0 {
            return 0;
        }
        let off = scroll_core::render_offset(*state, dir);
        let (pos, extent) = if dir == ScrollDirection::Vertical {
            (off.y, viewport.height as i32)
        } else {
            (off.x, viewport.width as i32)
        };
        let extent = extent.max(1);
        let idx = (pos + extent / 2).div_euclid(extent);
        idx.clamp(0, count as i32 - 1) as usize
    }

    /// Build the [`on_change`](Tileview::on_change) message for the active tile
    /// when it differs from `previous`. Returns `None` when no callback is set
    /// or the tile is unchanged. The host calls this from `update()` after
    /// applying a [`ScrollMsg`]/`tick`.
    #[must_use]
    pub fn change_msg(&self, previous: usize) -> Option<M> {
        let now = self.current_index();
        if now != previous {
            self.on_change.as_ref().map(|f| f(now))
        } else {
            None
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Tileview<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Tileview<'a, C, M> {
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
        let n = self.tiles.len() as u32;
        let vertical = self.dir == ScrollDirection::Vertical;
        self.content = if vertical {
            Size::new(rect.size.width, rect.size.height.saturating_mul(n))
        } else {
            Size::new(rect.size.width.saturating_mul(n), rect.size.height)
        };

        // Each tile fills the viewport; lay them edge-to-edge along the paging
        // axis, shifted by the rendered scroll offset.
        let off = scroll_core::render_offset(self.state, self.dir);
        let base = rect.top_left - off;
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            let origin = if vertical {
                Point::new(base.x, base.y + i as i32 * rect.size.height as i32)
            } else {
                Point::new(base.x + i as i32 * rect.size.width as i32, base.y)
            };
            tile.arrange(Rectangle::new(origin, rect.size));
        }
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        if self.tiles.is_empty() {
            return None;
        }
        let viewport = self.rect;
        let content = self.content;
        let dir = self.dir;
        let lines = self.snap_lines();
        let on_scroll = self.on_scroll.as_deref();
        let tiles = &mut self.tiles;
        scroll_core::route_touch(
            self.state,
            dir,
            viewport,
            content,
            point,
            phase,
            &lines,
            on_scroll,
            |p, ph| {
                // Forward to the active tile only (the others are off-screen),
                // top-most-last like other containers.
                for tile in tiles.iter_mut().rev() {
                    if scroll_core::rect_contains(tile.rect(), p) {
                        if let Some(msg) = tile.handle_touch(p, ph) {
                            return Some(msg);
                        }
                    }
                }
                None
            },
        )
    }

    fn mark_pressed(&mut self, point: Point) {
        // While dragging/animating, stop re-asserting child presses so a tile
        // control highlighted on Down is cancelled mid-swipe.
        if matches!(
            self.state.phase,
            zest_core::GesturePhase::Dragging
                | zest_core::GesturePhase::Flinging
                | zest_core::GesturePhase::Springing
        ) {
            return;
        }
        for tile in &mut self.tiles {
            tile.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let viewport = self.rect;
        renderer.push_clip(viewport);
        for tile in &self.tiles {
            // Skip tiles fully outside the viewport (only the current and an
            // adjacent in-transit tile ever overlap).
            if rects_overlap(tile.rect(), viewport) {
                tile.draw(renderer, theme)?;
            }
        }
        renderer.pop_clip();
        Ok(())
    }
}

/// Whether two rectangles share any area (half-open).
fn rects_overlap(a: Rectangle, b: Rectangle) -> bool {
    let a_br = a.top_left + Point::new(a.size.width as i32, a.size.height as i32);
    let b_br = b.top_left + Point::new(b.size.width as i32, b.size.height as i32);
    a.top_left.x < b_br.x && b.top_left.x < a_br.x && a.top_left.y < b_br.y && b.top_left.y < a_br.y
}
