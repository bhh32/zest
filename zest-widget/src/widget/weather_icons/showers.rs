use super::{cloudy, sunny};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    // Peek sun up-left of center (partly hidden by the cloud), then the same
    // lifted cloud + drops layout as `rain`, so showers shares the others'
    // center and its cloud lines up with the plain cloud / rain.
    sunny::draw_small(renderer, rect)?;

    let (cx, cy, size) = super::anchor(rect);
    let cloud_cy = cy - size / 12;
    cloudy::draw_at(renderer, Point::new(cx, cloud_cy), size)?;

    let drop_y = cloud_cy + size / 4;
    let color = Rgb565::CSS_DEEP_SKY_BLUE;
    for offset in [-size / 6, size / 6] {
        let x = cx + offset;
        renderer.stroke_line(
            Point::new(x, drop_y),
            Point::new(x - 2, drop_y + size / 6),
            color,
            2,
        )?;
    }

    Ok(())
}
