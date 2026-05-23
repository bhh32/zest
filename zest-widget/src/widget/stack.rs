//! Overlay container (a.k.a. ZStack / Layer). Holds children as
//! `Vec<Element>` and stacks them on the same region:
//!
//! * **Draw order** is insertion order — the first child is painted at the
//!   bottom, the last child on top.
//! * **Touch routing** is top-first — children are polled in reverse
//!   insertion order, so an overlay (modal, dropdown, menu) placed last
//!   intercepts input before the widgets beneath it.
//! * **Positioning** is per-child alignment: each child is measured against
//!   the stack's region and placed at its requested
//!   [`Horizontal`] / [`Vertical`] alignment (default center). A child that
//!   asks to fill simply covers the whole stack.
//!
//! This is the base for future overlay widgets (Dropdown, MessageBox,
//! Menu, Tooltip), so it is intentionally general.

use super::{Widget, element::Element};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Horizontal, Length, RenderError, Renderer, TouchPhase, Vertical};
use zest_theme::Theme;

/// A single layer in a [`Stack`]: a child plus the alignment used to
/// position it within the stack's region.
struct Layer<'a, C: PixelColor, M: Clone> {
    child: Element<'a, C, M>,
    align_x: Horizontal,
    align_y: Vertical,
}

/// Overlay container. Children share one region; later children draw on
/// top of and receive touches before earlier ones.
pub struct Stack<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    layers: Vec<Layer<'a, C, M>>,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Stack<'a, C, M> {
    /// Create a new empty stack. Position and size are assigned by the
    /// parent via `arrange`; use `.width(...)` / `.height(...)` to
    /// constrain the slot.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            layers: Vec::new(),
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Builder: push a child centered within the stack. Drawn on top of
    /// (and polled for touch before) every child pushed earlier.
    #[must_use]
    pub fn push<W>(self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.push_aligned(child, Horizontal::Center, Vertical::Center)
    }

    /// Builder: push a child positioned at an explicit alignment within
    /// the stack's region.
    #[must_use]
    pub fn push_aligned<W>(mut self, child: W, align_x: Horizontal, align_y: Vertical) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.layers.push(Layer {
            child: Element::new(child),
            align_x,
            align_y,
        });
        self
    }

    fn relayout(&mut self) {
        let outer = Constraints::loose(self.rect.size);
        let origin = self.rect.top_left;
        let avail = self.rect.size;
        for layer in &mut self.layers {
            let desired = layer.child.measure(outer);
            let cell = align_rect(origin, avail, desired, layer.align_x, layer.align_y);
            layer.child.arrange(cell);
        }
    }
}

/// Place a `desired`-sized rect within `[origin, origin + avail)` according
/// to the given alignments, clamping the size to the available region.
fn align_rect(
    origin: Point,
    avail: Size,
    desired: Size,
    align_x: Horizontal,
    align_y: Vertical,
) -> Rectangle {
    let w = desired.width.min(avail.width);
    let h = desired.height.min(avail.height);
    let free_x = avail.width.saturating_sub(w) as i32;
    let free_y = avail.height.saturating_sub(h) as i32;
    let dx = match align_x {
        Horizontal::Left => 0,
        Horizontal::Center => free_x / 2,
        Horizontal::Right => free_x,
    };
    let dy = match align_y {
        Vertical::Top => 0,
        Vertical::Center => free_y / 2,
        Vertical::Bottom => free_y,
    };
    Rectangle::new(origin + Point::new(dx, dy), Size::new(w, h))
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Stack<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Stack<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        // Intrinsic size is the max of all children's desired sizes.
        let mut intrinsic = Size::zero();
        let loose = Constraints::loose(constraints.max);
        for layer in &mut self.layers {
            let s = layer.child.measure(loose);
            intrinsic = Size::new(intrinsic.width.max(s.width), intrinsic.height.max(s.height));
        }
        let w = self.width.resolve(intrinsic.width, constraints.max.width);
        let h = self.height.resolve(intrinsic.height, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
        self.relayout();
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        // Top-first: the last-pushed (topmost) layer gets first refusal.
        for layer in self.layers.iter_mut().rev() {
            if let Some(msg) = layer.child.handle_touch(point, phase) {
                return Some(msg);
            }
        }
        None
    }

    fn mark_pressed(&mut self, point: Point) {
        for layer in &mut self.layers {
            layer.child.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        // Bottom-to-top: insertion order, last child on top.
        for layer in &self.layers {
            layer.child.draw(renderer, theme)?;
        }
        Ok(())
    }
}
