use super::Widget;
use crate::{Constraints, Length, RenderError, Renderer, TouchPhase};
use alloc::boxed::Box;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_theme::Theme;

/// Heterogeneous boxed widget. `'a` is the lifetime of any data the
/// inner widget borrows from screen state.
pub struct Element<'a, C: PixelColor, M: Clone> {
    inner: Box<dyn Widget<C, M> + 'a>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Element<'a, C, M> {
    /// Wrap a concrete widget.
    pub fn new<W>(widget: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        Self {
            inner: Box::new(widget),
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Element<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        self.inner.measure(constraints)
    }

    fn preferred_size(&self) -> (Length, Length) {
        self.inner.preferred_size()
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.inner.arrange(rect);
    }

    fn rect(&self) -> Rectangle {
        self.inner.rect()
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.inner.handle_touch(point, phase)
    }

    fn mark_pressed(&mut self, point: Point) {
        self.inner.mark_pressed(point);
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        self.inner.draw(renderer, theme)
    }
}

/// Conversion from a concrete widget into an [`Element`].
pub trait IntoElement<'a, C: PixelColor + 'a, M: Clone + 'a> {
    /// Wrap `self`.
    fn into_element(self) -> Element<'a, C, M>;
}

impl<'a, C, M, W> IntoElement<'a, C, M> for W
where
    C: PixelColor + 'a,
    M: Clone + 'a,
    W: Widget<C, M> + 'a,
{
    fn into_element(self) -> Element<'a, C, M> {
        Element::new(self)
    }
}
