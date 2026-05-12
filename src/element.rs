//! Heterogeneous widget container; wraps `Box<dyn Widget>`.

use crate::{Renderer, Theme, Widget, renderer::RenderError};
use alloc::boxed::Box;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};

/// A boxed widget. Containers (Column, Row, Grid, etc.) hold `Vec<Element>` so
/// children can be different concrete types.
pub struct Element<'a, C: PixelColor, M: Clone> {
    inner: Box<dyn Widget<'a, C, M> + 'a>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Element<'a, C, M> {
    /// Wrap a concrete widget.
    pub fn new<W>(widget: W) -> Self
    where
        W: Widget<'a, C, M> + 'a,
    {
        Self {
            inner: Box::new(widget),
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<'a, C, M> for Element<'a, C, M> {
    fn rect(&self) -> Rectangle {
        self.inner.rect()
    }

    fn set_rect(&mut self, rect: Rectangle) {
        self.inner.set_rect(rect);
    }

    fn handle_touch(&mut self, point: Point) -> Option<M> {
        self.inner.handle_touch(point)
    }

    fn draw(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'a, C>,
    ) -> Result<(), RenderError> {
        self.inner.draw(renderer, theme)
    }
}

pub trait IntoElement<'a, C: PixelColor + 'a, M: Clone + 'a> {
    /// Wrap this widget in an `Element`.
    fn into_element(self) -> Element<'a, C, M>;
}

impl<'a, C, M, W> IntoElement<'a, C, M> for W
where
    C: PixelColor + 'a,
    M: Clone + 'a,
    W: Widget<'a, C, M> + 'a,
{
    fn into_element(self) -> Element<'a, C, M> {
        Element::new(self)
    }
}
