use core::f32::consts::FRAC_1_SQRT_2;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

/// `(cos, sin)` for the eight ray angles `n * π/4`, `n = 0..8`.
///
/// Precomputed so this module needs no `f32` trig (which would pull in a
/// `micromath`/`libm` dependency on `no_std` targets).
const RAY_DIRS: [(f32, f32); 8] = [
    (1.0, 0.0),
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (0.0, 1.0),
    (-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (-1.0, 0.0),
    (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (0.0, -1.0),
    (FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
];

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    let center = Point::new(cx, cy);
    let body_ray = size / 4;

    renderer.fill_circle(center, body_ray as u32, Rgb565::CSS_GOLD)?;

    for &(cos, sin) in &RAY_DIRS {
        let start = center
            + Point::new(
                (cos * (body_ray + 2) as f32) as i32,
                (sin * (body_ray + 2) as f32) as i32,
            );
        let end = center
            + Point::new(
                (cos * (size / 2 - 2) as f32) as i32,
                (sin * (size / 2 - 2) as f32) as i32,
            );
        renderer.stroke_line(start, end, Rgb565::CSS_GOLD, 2)?;
    }

    Ok(())
}

pub(crate) fn draw_small(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    // A smaller sun nudged up-left of the rect's center, so an overlapping
    // cloud (PartlyCloudy / Showers) reveals it peeking out while the whole
    // glyph stays centered. The offset is relative to the shared centered
    // anchor, not the rect's top-left.
    let (cx, cy, size) = super::anchor(rect);
    // Half-size sun nudged up-left of center — small enough that, with the
    // overlapping cloud, the composition stays inside the rect (no top
    // clipping) while still reading as a sun peeking out.
    let edge = size / 2;
    let scx = cx - size / 6;
    let scy = cy - size / 6;
    let sun = Rectangle::new(
        Point::new(scx - edge / 2, scy - edge / 2),
        Size::new(edge as u32, edge as u32),
    );
    draw(renderer, sun)
}
