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
    let center =
        rect.top_left + Point::new(rect.size.width as i32 / 2, rect.size.height as i32 / 2);
    let size = rect.size.width.min(rect.size.height) as i32;
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
    let offset_rect = Rectangle::new(
        rect.top_left - Point::new(0, rect.size.height as i32 / 8),
        rect.size * 7 / 10,
    );
    draw(renderer, offset_rect)
}
