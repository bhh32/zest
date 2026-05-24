use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    let card_x = cx;
    let color = Rgb565::CSS_LIGHT_GRAY;

    for (i, y_frac) in [3, 5, 7].iter().enumerate() {
        let y = cy + (*y_frac - 5) * size / 10;
        let pad = if i == 1 { size / 12 } else { size / 6 };
        renderer.stroke_line(
            Point::new(card_x - size / 2 + pad, y),
            Point::new(card_x + size / 2 - pad, y),
            color,
            2,
        )?;
    }

    Ok(())
}
