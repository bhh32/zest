use super::cloudy;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{RenderError, Renderer};

pub(crate) fn draw(
    renderer: &mut dyn Renderer<Rgb565>,
    rect: Rectangle,
) -> Result<(), RenderError> {
    let (cx, cy, size) = super::anchor(rect);
    // The bolt is the tallest precipitation, so lift the cloud by half its
    // height to keep cloud + bolt centered on (cx, cy).
    let cloud_cy = cy - size / 8;
    cloudy::draw_at(renderer, Point::new(cx, cloud_cy), size)?;

    let card_x = cx;
    let bolt_top = cloud_cy + size / 4;
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
