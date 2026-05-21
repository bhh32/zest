//! Static raster image. Blits a borrowed slice of pixels into its arranged
//! rect via [`Renderer::draw_image`] (zest_core::Renderer::draw_image),
//! centerin when the slot is larget than the image.
//!
//! The pixel buffer is borrowed (`&'a [C]`), so the owner, typically a
//! screen holding a decoded frame, keeps it alive for the frame.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Raster image drawn from a borrowed, row-major pixel slice.
pub struct Image<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    pixels: &'a [C],
    image_size: Size,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor, M: Clone> Image<'a, C, M> {
    /// New image froma row-major `pixels` slice of `image_size`. Defaults to
    /// shrinking to the image's intrinsic size; position is assigned by the
    /// parent via `arrange`.
    pub fn new(pixels: &'a [C], image_size: Size) -> Self {
        Self {
            rect: Rectangle::zero(),
            pixels,
            image_size,
            width: Length::Shrink,
            height: Length::Shrink,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Image<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let width = self
            .width
            .resolve(self.image_size.width, constraints.max.width);
        let height = self
            .height
            .resolve(self.image_size.height, constraints.max.height);
        constraints.clamp(Size::new(width, height))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, _point: Point, _phase: TouchPhase) -> Option<M> {
        None
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        _theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let dx = (self.rect.size.width as i32 - self.image_size.width as i32).max(0) / 2;
        let dy = (self.rect.size.height as i32 - self.image_size.height as i32).max(0) / 2;
        let origin = self.rect.top_left + Point::new(dx, dy);
        renderer.draw_image(origin, self.image_size, self.pixels)
    }
}
