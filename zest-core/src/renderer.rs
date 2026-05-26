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
    fn draw_image(
        &mut self,
        _top_left: Point,
        _size: Size,
        _pixels: &[C],
    ) -> Result<(), RenderError> {
        Ok(())
    }

    /// Stroke a circular arc centered at `center` with the given
    /// `radius`, beginning at `start_deg` and sweeping `sweep_deg`
    /// degrees (positive = counter-clockwise in screen space, i.e.
    /// toward the top, since 0° points right). `width` is the stroke
    /// thickness in pixels.
    ///
    /// The default implementation approximates the arc with short
    /// [`stroke_line`](Self::stroke_line) segments using the trig-free
    /// per-degree [`sin/cos lookup table`](arc_sin_cos), so it works on
    /// any backend — including the headless default — without pulling in
    /// floating-point trig. Backends with native arc/path support
    /// (e.g. the tiny-skia simulator) override this for anti-aliased
    /// output.
    fn stroke_arc(
        &mut self,
        center: Point,
        radius: u32,
        start_deg: i32,
        sweep_deg: i32,
        width: u32,
        color: C,
    ) -> Result<(), RenderError> {
        if radius == 0 || width == 0 || sweep_deg == 0 {
            return Ok(());
        }
        // One segment per degree keeps the polyline smooth at the radii
        // typical for embedded displays while staying cheap. Clamp the
        // sweep to a full turn so a huge value can't loop forever.
        let total = sweep_deg.unsigned_abs().min(360);
        let step: i32 = if sweep_deg >= 0 { 1 } else { -1 };
        let r = radius as f32;

        let point_at = |deg: i32| -> Point {
            let (s, c) = arc_sin_cos(deg);
            // Screen y grows downward, so subtract the sine to make a
            // positive sweep travel counter-clockwise visually.
            Point::new(center.x + (c * r) as i32, center.y - (s * r) as i32)
        };

        let mut prev = point_at(start_deg);
        for i in 1..=total as i32 {
            let next = point_at(start_deg + i * step);
            self.stroke_line(prev, next, color, width)?;
            prev = next;
        }
        Ok(())
    }

    /// Fill a circular sector (pie slice) centered at `center` with the
    /// given `radius`, from `start_deg` sweeping `sweep_deg` degrees.
    ///
    /// The default implementation rasterizes the sector as a fan of
    /// triangle-like spokes drawn with [`stroke_line`](Self::stroke_line)
    /// from the center to each arc point — trig-free via
    /// [`arc_sin_cos`]. Backends with native fill support override this.
    fn fill_arc(
        &mut self,
        center: Point,
        radius: u32,
        start_deg: i32,
        sweep_deg: i32,
        color: C,
    ) -> Result<(), RenderError> {
        if radius == 0 || sweep_deg == 0 {
            return Ok(());
        }
        let total = sweep_deg.unsigned_abs().min(360);
        let step: i32 = if sweep_deg >= 0 { 1 } else { -1 };
        let r = radius as f32;
        // Spoke width 2 closes the seams between adjacent spokes at the
        // rim without the gaps a 1px line would leave.
        for i in 0..=total as i32 {
            let (s, c) = arc_sin_cos(start_deg + i * step);
            let end = Point::new(center.x + (c * r) as i32, center.y - (s * r) as i32);
            self.stroke_line(center, end, color, 2)?;
        }
        Ok(())
    }

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

/// Per-degree sine lookup, index `0..360` ⇒ `sin(deg)`.
///
/// Precomputed so [`Renderer::stroke_arc`] and [`Renderer::fill_arc`]
/// stay trig-free in `no_std` (no `libm` dependency). Cosine is obtained
/// from the same table via the identity `cos(x) = sin(x + 90°)`.
static SIN_TABLE: [f32; 360] = [
    0.0,
    0.01745241,
    0.0348995,
    0.05233596,
    0.06975647,
    0.08715574,
    0.1045285,
    0.1218693,
    0.1391731,
    0.1564345,
    0.1736482,
    0.190809,
    0.2079117,
    0.2249511,
    0.2419219,
    0.258819,
    0.2756374,
    0.2923717,
    0.309017,
    0.3255682,
    0.3420201,
    0.3583679,
    0.3746066,
    0.3907311,
    0.4067366,
    0.4226183,
    0.4383711,
    0.4539905,
    0.4694716,
    0.4848096,
    0.5,
    0.5150381,
    0.5299193,
    0.544639,
    0.5591929,
    0.5735764,
    0.5877853,
    0.601815,
    0.6156615,
    0.6293204,
    0.6427876,
    0.656059,
    0.6691306,
    0.6819984,
    0.6946584,
    0.7071068,
    0.7193398,
    0.7313537,
    0.7431448,
    0.7547096,
    0.7660444,
    0.777146,
    0.7880108,
    0.7986355,
    0.809017,
    0.819152,
    0.8290376,
    0.8386706,
    0.8480481,
    0.8571673,
    0.8660254,
    0.8746197,
    0.8829476,
    0.8910065,
    0.898794,
    0.9063078,
    0.9135455,
    0.9205049,
    0.9271839,
    0.9335804,
    0.9396926,
    0.9455186,
    0.9510565,
    0.9563048,
    0.9612617,
    0.9659258,
    0.9702957,
    0.9743701,
    0.9781476,
    0.9816272,
    0.9848078,
    0.9876883,
    0.9902681,
    0.9925462,
    0.9945219,
    0.9961947,
    0.9975641,
    0.9986295,
    0.9993908,
    0.9998477,
    1.0,
    0.9998477,
    0.9993908,
    0.9986295,
    0.9975641,
    0.9961947,
    0.9945219,
    0.9925462,
    0.9902681,
    0.9876883,
    0.9848078,
    0.9816272,
    0.9781476,
    0.9743701,
    0.9702957,
    0.9659258,
    0.9612617,
    0.9563048,
    0.9510565,
    0.9455186,
    0.9396926,
    0.9335804,
    0.9271839,
    0.9205049,
    0.9135455,
    0.9063078,
    0.898794,
    0.8910065,
    0.8829476,
    0.8746197,
    0.8660254,
    0.8571673,
    0.8480481,
    0.8386706,
    0.8290376,
    0.819152,
    0.809017,
    0.7986355,
    0.7880108,
    0.777146,
    0.7660444,
    0.7547096,
    0.7431448,
    0.7313537,
    0.7193398,
    0.7071068,
    0.6946584,
    0.6819984,
    0.6691306,
    0.656059,
    0.6427876,
    0.6293204,
    0.6156615,
    0.601815,
    0.5877853,
    0.5735764,
    0.5591929,
    0.544639,
    0.5299193,
    0.5150381,
    0.5,
    0.4848096,
    0.4694716,
    0.4539905,
    0.4383711,
    0.4226183,
    0.4067366,
    0.3907311,
    0.3746066,
    0.3583679,
    0.3420201,
    0.3255682,
    0.309017,
    0.2923717,
    0.2756374,
    0.258819,
    0.2419219,
    0.2249511,
    0.2079117,
    0.190809,
    0.1736482,
    0.1564345,
    0.1391731,
    0.1218693,
    0.1045285,
    0.08715574,
    0.06975647,
    0.05233596,
    0.0348995,
    0.01745241,
    0.0,
    -0.01745241,
    -0.0348995,
    -0.05233596,
    -0.06975647,
    -0.08715574,
    -0.1045285,
    -0.1218693,
    -0.1391731,
    -0.1564345,
    -0.1736482,
    -0.190809,
    -0.2079117,
    -0.2249511,
    -0.2419219,
    -0.258819,
    -0.2756374,
    -0.2923717,
    -0.309017,
    -0.3255682,
    -0.3420201,
    -0.3583679,
    -0.3746066,
    -0.3907311,
    -0.4067366,
    -0.4226183,
    -0.4383711,
    -0.4539905,
    -0.4694716,
    -0.4848096,
    -0.5,
    -0.5150381,
    -0.5299193,
    -0.544639,
    -0.5591929,
    -0.5735764,
    -0.5877853,
    -0.601815,
    -0.6156615,
    -0.6293204,
    -0.6427876,
    -0.656059,
    -0.6691306,
    -0.6819984,
    -0.6946584,
    -0.7071068,
    -0.7193398,
    -0.7313537,
    -0.7431448,
    -0.7547096,
    -0.7660444,
    -0.777146,
    -0.7880108,
    -0.7986355,
    -0.809017,
    -0.819152,
    -0.8290376,
    -0.8386706,
    -0.8480481,
    -0.8571673,
    -0.8660254,
    -0.8746197,
    -0.8829476,
    -0.8910065,
    -0.898794,
    -0.9063078,
    -0.9135455,
    -0.9205049,
    -0.9271839,
    -0.9335804,
    -0.9396926,
    -0.9455186,
    -0.9510565,
    -0.9563048,
    -0.9612617,
    -0.9659258,
    -0.9702957,
    -0.9743701,
    -0.9781476,
    -0.9816272,
    -0.9848078,
    -0.9876883,
    -0.9902681,
    -0.9925462,
    -0.9945219,
    -0.9961947,
    -0.9975641,
    -0.9986295,
    -0.9993908,
    -0.9998477,
    -1.0,
    -0.9998477,
    -0.9993908,
    -0.9986295,
    -0.9975641,
    -0.9961947,
    -0.9945219,
    -0.9925462,
    -0.9902681,
    -0.9876883,
    -0.9848078,
    -0.9816272,
    -0.9781476,
    -0.9743701,
    -0.9702957,
    -0.9659258,
    -0.9612617,
    -0.9563048,
    -0.9510565,
    -0.9455186,
    -0.9396926,
    -0.9335804,
    -0.9271839,
    -0.9205049,
    -0.9135455,
    -0.9063078,
    -0.898794,
    -0.8910065,
    -0.8829476,
    -0.8746197,
    -0.8660254,
    -0.8571673,
    -0.8480481,
    -0.8386706,
    -0.8290376,
    -0.819152,
    -0.809017,
    -0.7986355,
    -0.7880108,
    -0.777146,
    -0.7660444,
    -0.7547096,
    -0.7431448,
    -0.7313537,
    -0.7193398,
    -0.7071068,
    -0.6946584,
    -0.6819984,
    -0.6691306,
    -0.656059,
    -0.6427876,
    -0.6293204,
    -0.6156615,
    -0.601815,
    -0.5877853,
    -0.5735764,
    -0.5591929,
    -0.544639,
    -0.5299193,
    -0.5150381,
    -0.5,
    -0.4848096,
    -0.4694716,
    -0.4539905,
    -0.4383711,
    -0.4226183,
    -0.4067366,
    -0.3907311,
    -0.3746066,
    -0.3583679,
    -0.3420201,
    -0.3255682,
    -0.309017,
    -0.2923717,
    -0.2756374,
    -0.258819,
    -0.2419219,
    -0.2249511,
    -0.2079117,
    -0.190809,
    -0.1736482,
    -0.1564345,
    -0.1391731,
    -0.1218693,
    -0.1045285,
    -0.08715574,
    -0.06975647,
    -0.05233596,
    -0.0348995,
    -0.01745241,
];

/// Trig-free `(sin, cos)` of `deg` degrees via a precomputed sine table.
///
/// Accepts any integer degree (negative or > 360); it is reduced modulo
/// 360 first. This is the primitive the default arc implementations use
/// so `zest-core` needs no `libm`/`std` math.
#[must_use]
pub fn arc_sin_cos(deg: i32) -> (f32, f32) {
    let d = deg.rem_euclid(360) as usize;
    let c = (deg + 90).rem_euclid(360) as usize;
    (SIN_TABLE[d], SIN_TABLE[c])
}
