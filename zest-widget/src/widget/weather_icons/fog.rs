use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let size = rect.size.width.min(rect.size.height) as i32;
    let card_x = rect.top_left.x + size / 2;
    let color = Rgb565::CSS_LIGHT_GRAY;

    for (i, y_frac) in [3, 5, 7].iter().enumerate() {
        let y = rect.top_left.y + size * y_frac / 10;
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
