use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub fn draw(renderer: &mut dyn Renderer<Rgb565>, rect: Rectangle) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    draw_at(renderer, Point::new(cx, cy), size)
}

/// Draw a cloud centered at `center`, scaled to `size`. The precipitation
/// glyphs call this directly so they can lift the cloud and hang drops / a
/// bolt below it while keeping the whole glyph centered on the shared anchor.
pub(crate) fn draw_at(
    renderer: &mut dyn Renderer<Rgb565>,
    center: Point,
    size: i32,
) -> Result<(), RenderError> {
    let big_radius = size / 4;
    let small_radius = size / 5;
    let color = Rgb565::CSS_LIGHT_GRAY;

    renderer.fill_circle(center, big_radius as u32, color)?;
    renderer.fill_circle(center - Point::new(big_radius, 0), small_radius as u32, color)?;
    renderer.fill_circle(center + Point::new(big_radius, 0), small_radius as u32, color)?;

    let rect_width = (big_radius * 2 + small_radius * 2) as u32;
    let bar = Rectangle::new(
        center - Point::new(big_radius + small_radius, 0),
        Size::new(rect_width, (size / 8) as u32),
    );
    renderer.fill_rect(bar, color)?;

    Ok(())
}
