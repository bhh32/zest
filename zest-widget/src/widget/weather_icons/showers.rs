use super::{cloudy, sunny};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    sunny::draw_small(renderer, rect)?;

    let size = rect.size.width.min(rect.size.height) as i32;
    let cloud_rect = Rectangle::new(
        rect.top_left + Point::new(size / 4, size / 8),
        Size::new(rect.size.width * 3 / 4, rect.size.height * 3 / 4),
    );
    cloudy::draw(renderer, cloud_rect)?;

    let cloud_size = cloud_rect.size.width.min(cloud_rect.size.height) as i32;
    let drop_y = cloud_rect.top_left.y + cloud_size * 3 / 4;
    let color = Rgb565::CSS_DEEP_SKY_BLUE;

    for offset in [-cloud_size / 6, cloud_size / 6] {
        let x = cloud_rect.top_left.x + cloud_size / 2 + offset;
        renderer.stroke_line(
            Point::new(x, drop_y),
            Point::new(x - 2, drop_y + cloud_size / 6),
            color,
            2,
        )?;
    }

    Ok(())
}
