use super::cloudy;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    // Flakes are small, so only a slight lift keeps the glyph centered.
    let cloud_cy = cy - size / 24;
    cloudy::draw_at(renderer, Point::new(cx, cloud_cy), size)?;

    let flake_y = cloud_cy + size / 4;
    let color = Rgb565::WHITE;
    for offset in [-size / 6, 0, size / 6] {
        let x = cx + offset;
        renderer.fill_circle(Point::new(x, flake_y), 2, color)?;
    }

    Ok(())
}
