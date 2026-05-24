use embedded_graphics::{
    mono_font::iso_8859_15::FONT_10X20, pixelcolor::Rgb565, prelude::*, primitives::Rectangle,
    text::Alignment,
};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    let card_x = cx;
    let card_y = cy;
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
