use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub fn draw(renderer: &mut dyn Renderer<Rgb565>, rect: Rectangle) -> Result<(), RenderError> {
    let size = rect.size.width.min(rect.size.height) as i32;
    let center = rect.top_left + Point::new(size / 2, size * 5 / 8);
    let big_radius = size / 4;
    let small_radius = size / 5;
    let color = Rgb565::CSS_LIGHT_GRAY;

    renderer.fill_circle(center, big_radius as u32, color)?;
    renderer.fill_circle(
        center - Point::new(big_radius, 0),
        small_radius as u32,
        color,
    )?;
    renderer.fill_circle(
        center + Point::new(big_radius, 0),
        small_radius as u32,
        color,
    )?;

    let rect_width = (big_radius * 2 + small_radius * 2) as u32;
    let bar = Rectangle::new(
        center - Point::new(big_radius + small_radius, 0),
        Size::new(rect_width, (size / 8) as u32),
    );
    renderer.fill_rect(bar, color)?;

    Ok(())
}
