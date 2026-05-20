use super::cloudy;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub fn draw(renderer: &mut dyn Renderer<Rgb565>, rect: Rectangle) -> Result<(), RenderError> {
    cloudy::draw(renderer, rect)?;

    let size = rect.size.width.min(rect.size.height) as i32;
    let drop_y = rect.top_left.y + size * 3 / 4;
    let color = Rgb565::CSS_DEEP_SKY_BLUE;

    for offset in [-size / 6, 0, size / 6] {
        let x = rect.top_left.x + size / 2 + offset;

        renderer.stroke_line(
            Point::new(x, drop_y),
            Point::new(x - 2, drop_y + size / 6),
            color,
            2,
        )?;
    }

    Ok(())
}
