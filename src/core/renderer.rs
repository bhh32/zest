//! Object-safe renderer abstraction so the Widget trait can be `dyn`.
//!
//! The `Renderer` trait wraps the operations every widget needs from a
//! `DrawTarget`. `DrawTargetRenderer` is the standard adapter from any
//! `embedded_graphics::DrawTarget` to `Renderer`.

use core::fmt;
use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::PixelColor,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    text::{Alignment, Text},
};

/// Errors produced by Renderer operations. The underlying `DrawTarget::Error`
/// is erased; widgets that need detail shoud use a custom Renderer impl.
pub struct RenderError;

impl<E: fmt::Debug> From<E> for RenderError {
    fn from(_: E) -> Self {
        RenderError
    }
}

// Help the blanket impl above not conflict with itself.

/// Object-safe rendering operations. Widgets call these instead of touching
/// `DrawTarget` directly, which lets `Widget` be a trait object.
pub trait Renderer<C: PixelColor> {
    /// Fill a rectangle with solid color.
    fn fill_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError>;
    /// Draw a 1-pixel border (inside-aligned) around a rectangle.
    fn stroke_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError>;
    /// Render text with a mono font, anchored at `position` with `alignment`.
    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font: &MonoFont<'_>,
        color: C,
        alignment: Alignment,
    ) -> Result<(), RenderError>;
}

/// Adapter that implements `Renderer` over any `embedded_graphics::DrawTarget`.
pub struct DrawTargetRenderer<'d, D> {
    display: &'d mut D,
}

impl<'d, D> DrawTargetRenderer<'d, D> {
    /// Create a renderer that delegates to `display`.
    pub fn new(display: &'d mut D) -> Self {
        Self { display }
    }
}

impl<'d, C, D> Renderer<C> for DrawTargetRenderer<'d, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    fn fill_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError> {
        rect.into_styled(PrimitiveStyle::with_fill(color))
            .draw(self.display)
            .map_err(|_| RenderError)
    }

    fn stroke_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError> {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        rect.into_styled(style)
            .draw(self.display)
            .map_err(|_| RenderError)
    }

    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font: &MonoFont<'_>,
        color: C,
        alignment: Alignment,
    ) -> Result<(), RenderError> {
        let style = MonoTextStyle::new(font, color);
        Text::with_alignment(text, position, style, alignment)
            .draw(self.display)
            .map(|_| ())
            .map_err(|_| RenderError)
    }
}
