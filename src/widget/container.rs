//! Bounds-aware single-child wrapper. Applies padding and fowards
//! `handle_touch`/`draw` to its inner widget.

use crate::{Element, Renderer, Theme, Widget, renderer::RenderError};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};

/// Holds one child inside a bounded region with optional padding.
pub struct Container<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    padding: u32,
    child: Option<Element<'a, C, M>>,
}

impl<'a, C: PixelColor, M: Clone> Container<'a, C, M> {
    /// Create a new empty contianer with bounds.
    pub fn new(rect: Rectangle) -> Self {
        Self {
            rect,
            padding: 0,
            child: None,
        }
    }

    /// Builder: padding inset on all sides.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Builder: set the child widget.
    pub fn child(mut self, child: impl Into<Element<'a, C, M>>) -> Self {
        let mut elem = child.into();
        let inner = self.inner_rect();
        elem.set_rect(inner);
        self.child = Some(elem);
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

impl<'a, C: PixelColor, M: Clone> Widget<'a, C, M> for Container<'a, C, M> {
    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn set_rect(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn handle_touch(&mut self, point: Point) -> Option<M> {
        self.child
            .as_mut()
            .and_then(|child| child.handle_touch(point))
    }

    fn draw(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'a, C>,
    ) -> Result<(), RenderError> {
        if let Some(child) = &self.child {
            child.draw(renderer, theme)?;
        }

        Ok(())
    }
}
