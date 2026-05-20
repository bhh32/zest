use super::cloudy;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    cloudy::draw(renderer, rect)?;

    let size = rect.size.width.min(rect.size.height) as i32;
    let card_x = rect.top_left.x + size / 2;
    let bolt_top = rect.top_left.y + size * 3 / 4;
    let color = Rgb565::CSS_GOLD;

    let p1 = Point::new(card_x + size / 12, bolt_top);
    let p2 = Point::new(card_x - size / 12, bolt_top + size / 8);
    let p3 = Point::new(card_x + size / 16, bolt_top + size / 8);
    let p4 = Point::new(card_x - size / 8, bolt_top + size / 4);

    renderer.stroke_line(p1, p2, color, 2)?;
    renderer.stroke_line(p2, p3, color, 2)?;
    renderer.stroke_line(p3, p4, color, 2)?;

    Ok(())
}
