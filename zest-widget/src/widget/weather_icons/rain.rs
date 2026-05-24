use super::cloudy;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub fn draw(renderer: &mut dyn Renderer<Rgb565>, rect: Rectangle) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    // Lift the cloud by half the drop length so cloud + drops together center
    // on (cx, cy) — the same anchor sunny/cloudy use.
    let cloud_cy = cy - size / 12;
    cloudy::draw_at(renderer, Point::new(cx, cloud_cy), size)?;

    let drop_y = cloud_cy + size / 4;
    let color = Rgb565::CSS_DEEP_SKY_BLUE;
    for offset in [-size / 6, 0, size / 6] {
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
