use embedded_graphics::{
    mono_font::iso_8859_15::FONT_10X20, pixelcolor::Rgb565, prelude::*, primitives::Rectangle,
    text::Alignment,
};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let size = rect.size.width.min(rect.size.height) as i32;
    let card_x = rect.top_left.x + size / 2;
    let card_y = rect.top_left.y + size / 2;
    let color = Rgb565::CSS_DIM_GRAY;

    renderer.fill_circle(Point::new(card_x, card_y), (size / 3) as u32, color)?;

    renderer.draw_text(
        "?",
        Point::new(card_x, card_y + size / 6),
        &FONT_10X20,
        Rgb565::CSS_LIGHT_GRAY,
        Alignment::Center,
    )?;

    Ok(())
}
