//! Bounds-aware single-child wrapper. Applies padding and forwards
//! the touch / draw protocol to its inner widget.

use super::{Widget, element::Element};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Holds one child inside a bounded region with optional padding.
pub struct Container<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    padding: u32,
    child: Option<Element<'a, C, M>>,
    width: Length,
    height: Length,
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
        }
    }

    /// Builder: padding inset on all sides.
    #[must_use]
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
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

    /// Builder: set the child widget. The parent will call `arrange`
    /// later, which propagates the inner rect to the child.
    #[must_use]
    pub fn child<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.child = Some(Element::new(child));
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
        let inner = self.inner_rect();
        if let Some(child) = self.child.as_mut() {
            child.arrange(inner);
        }
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.child
            .as_mut()
            .and_then(|child| child.handle_touch(point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(child) = self.child.as_mut() {
            child.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        if let Some(child) = &self.child {
            child.draw(renderer, theme)?;
        }
        Ok(())
    }
}
