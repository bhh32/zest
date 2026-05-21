//! Object-safe rendering abstraction over `embedded-graphics`'s [`DrawTarget`].
//!
//! The `Widget` trait must be object-safe so
//! `Element` can hold `Box<dyn Widget>`.
//! `DrawTarget` is generic over its error type, which prevents direct use
//! in trait objects. [`Renderer`] erases the error type into a single
//! [`RenderError`] and exposes only the operations widgets need.
//!
//! [`DrawTargetRenderer`] is the standard adapter from any concrete
//! `DrawTarget` to `Renderer`.

use core::fmt;
use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::PixelColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    text::{Alignment, Text},
};

/// Type-erased rendering error.
///
/// The underlying [`DrawTarget::Error`] is discarded. Widgets needing rich
/// error information should implement a custom [`Renderer`] that preserves it.
#[derive(Copy, Clone, Debug)]
pub struct RenderError;

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("render error")
    }
}

/// Object-safe rendering operations. Widgets call these instead of touching
/// [`DrawTarget`] directly, so `Widget` can be a
/// trait object.
pub trait Renderer<C: PixelColor> {
    /// Fill `rect` with a solid color.
    fn fill_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError>;
    /// Draw a 1-pixel inside-aligned border around `rect`.
    fn stroke_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError>;
    /// Fill circle with a solid color.
    fn fill_circle(&mut self, center: Point, radius: u32, color: C) -> Result<(), RenderError>;
    /// Create a stroke line.
    fn stroke_line(
        &mut self,
        start: Point,
        end: Point,
        color: C,
        width: u32,
    ) -> Result<(), RenderError>;
    /// Render text with a mono font at `position` with the given alignment.
    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font: &MonoFont<'_>,
        color: C,
        alignment: Alignment,
    ) -> Result<(), RenderError>;

    /// Used to draw an Image widget.
    fn draw_image(&mut self, top_left: Point, size: Size, pixels: &[C]) -> Result<(), RenderError>;

    /// Push a clipping rectangle. Subsequent draw calls are restricted
    /// to the intersection of all currently-pushed clip rects.
    ///
    /// Used by `Scrollable` and other viewport-aware widgets. Default
    /// implementation is a no-op (suitable for backends without
    /// clip support); the desktop tiny-skia backend implements it
    /// properly via masks.
    fn push_clip(&mut self, _rect: Rectangle) {}

    /// Pop the topmost clip rect previously pushed via
    /// [`push_clip`](Self::push_clip).
    fn pop_clip(&mut self) {}
}

/// Adapter implementing [`Renderer`] over any [`DrawTarget`].
pub struct DrawTargetRenderer<'d, D> {
    target: &'d mut D,
}

impl<'d, D> DrawTargetRenderer<'d, D> {
    /// Wrap a mutable reference to a `DrawTarget`.
    pub fn new(target: &'d mut D) -> Self {
        Self { target }
    }
}

impl<'d, C, D> Renderer<C> for DrawTargetRenderer<'d, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    fn fill_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError> {
        rect.into_styled(PrimitiveStyle::with_fill(color))
            .draw(self.target)
            .map_err(|_| RenderError)
    }

    fn stroke_rect(&mut self, rect: Rectangle, color: C) -> Result<(), RenderError> {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();
        rect.into_styled(style)
            .draw(self.target)
            .map_err(|_| RenderError)
    }

    fn fill_circle(&mut self, center: Point, radius: u32, color: C) -> Result<(), RenderError> {
        // `Circle::with_center` takes a diameter, so double here.
        Circle::with_center(center, radius * 2)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(self.target)
            .map_err(|_| RenderError)
    }

    fn stroke_line(
        &mut self,
        start: Point,
        end: Point,
        color: C,
        width: u32,
    ) -> Result<(), RenderError> {
        Line::new(start, end)
            .into_styled(PrimitiveStyle::with_stroke(color, width))
            .draw(self.target)
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
            .draw(self.target)
            .map(|_| ())
            .map_err(|_| RenderError)
    }

    fn draw_image(&mut self, top_left: Point, size: Size, pixels: &[C]) -> Result<(), RenderError> {
        let area = Rectangle::new(top_left, size);
        self.target
            .fill_contiguous(&area, pixels.iter().copied())
            .map_err(|_| RenderError)
    }
}
