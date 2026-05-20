use super::cloudy;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    cloudy::draw(renderer, rect)?;

    let size = rect.size.width.min(rect.size.height) as i32;
    let flake_y = rect.top_left.y + size * 3 / 4;
    let color = Rgb565::WHITE;

    for offset in [-size / 6, 0, size / 6] {
        let x = rect.top_left.x + size / 2 + offset;
        renderer.fill_circle(Point::new(x, flake_y), 2, color)?;
    }

    Ok(())
}
