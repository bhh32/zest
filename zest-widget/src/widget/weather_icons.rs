use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase, Widget};
use zest_theme::Theme;

mod cloudy;
mod fog;
mod rain;
mod showers;
mod snow;
mod sunny;
mod thunderstorm;
mod unknown;
mod windy;

/// Centered square drawing anchor for a glyph within `rect`.
///
/// Returns `(cx, cy, size)`: `(cx, cy)` is the rect's true center — computed
/// from its actual width/height, so a glyph stays centered in a
/// wider-than-tall (or taller-than-wide) rect — and `size` is the largest
/// square that fits. Every glyph anchors here and centers on `(cx, cy)`, so a
/// row of mixed conditions shares one visual center.
fn anchor(rect: Rectangle) -> (i32, i32, i32) {
    let size = rect.size.width.min(rect.size.height) as i32;
    let cx = rect.top_left.x + rect.size.width as i32 / 2;
    let cy = rect.top_left.y + rect.size.height as i32 / 2;
    (cx, cy, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::{mono_font::MonoFont, text::Alignment};

    #[derive(Default)]
    struct Bbox {
        minx: i32,
        miny: i32,
        maxx: i32,
        maxy: i32,
        any: bool,
    }
    impl Bbox {
        fn add(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
            if !self.any {
                (self.minx, self.miny, self.maxx, self.maxy, self.any) = (x0, y0, x1, y1, true);
            } else {
                self.minx = self.minx.min(x0);
                self.miny = self.miny.min(y0);
                self.maxx = self.maxx.max(x1);
                self.maxy = self.maxy.max(y1);
            }
        }
        fn cx(&self) -> i32 {
            (self.minx + self.maxx) / 2
        }
        fn cy(&self) -> i32 {
            (self.miny + self.maxy) / 2
        }
    }
    struct Rec(Bbox);
    impl Renderer<Rgb565> for Rec {
        fn fill_rect(&mut self, r: Rectangle, _: Rgb565) -> Result<(), RenderError> {
            self.0.add(
                r.top_left.x,
                r.top_left.y,
                r.top_left.x + r.size.width as i32,
                r.top_left.y + r.size.height as i32,
            );
            Ok(())
        }
        fn stroke_rect(&mut self, r: Rectangle, c: Rgb565) -> Result<(), RenderError> {
            self.fill_rect(r, c)
        }
        fn fill_circle(&mut self, ctr: Point, radius: u32, _: Rgb565) -> Result<(), RenderError> {
            let r = radius as i32;
            self.0.add(ctr.x - r, ctr.y - r, ctr.x + r, ctr.y + r);
            Ok(())
        }
        fn stroke_line(&mut self, a: Point, b: Point, _: Rgb565, w: u32) -> Result<(), RenderError> {
            let w = w as i32;
            self.0.add(a.x.min(b.x) - w, a.y.min(b.y) - w, a.x.max(b.x) + w, a.y.max(b.y) + w);
            Ok(())
        }
        fn draw_text(
            &mut self,
            text: &str,
            pos: Point,
            font: &MonoFont,
            _: Rgb565,
            _: Alignment,
        ) -> Result<(), RenderError> {
            let hw = text.chars().count() as i32 * font.character_size.width as i32 / 2;
            self.0
                .add(pos.x - hw, pos.y - font.baseline as i32, pos.x + hw, pos.y);
            Ok(())
        }
    }

    fn bb(f: impl FnOnce(&mut Rec), rect: Rectangle) -> Bbox {
        let mut r = Rec(Bbox::default());
        f(&mut r);
        r.0
    }

    /// Every glyph must center on the shared anchor and stay within the
    /// centered square, in both a square and a wide rect. Guards against the
    /// left-shift (non-square rects) and inconsistent-baseline regressions.
    #[test]
    fn glyphs_centered() {
        for rect in [
            Rectangle::new(Point::new(0, 0), Size::new(200, 80)),
            Rectangle::new(Point::new(0, 0), Size::new(80, 80)),
        ] {
            let (cx, cy, size) = anchor(rect);
            let half = size / 2;
            let check = |name: &str, b: Bbox, tol: i32| {
                assert!((b.cx() - cx).abs() <= 2, "{name}: cx={} want {cx}", b.cx());
                assert!(
                    (b.cy() - cy).abs() <= tol,
                    "{name}: cy={} want {cy}±{tol}",
                    b.cy()
                );
                assert!(
                    b.minx >= cx - half - 2
                        && b.maxx <= cx + half + 2
                        && b.miny >= cy - half - 2
                        && b.maxy <= cy + half + 2,
                    "{name}: bbox x[{}..{}] y[{}..{}] spills the centered square",
                    b.minx,
                    b.maxx,
                    b.miny,
                    b.maxy
                );
            };
            // Single-element glyphs share the anchor tightly.
            check("sunny", bb(|r| sunny::draw(r, rect).unwrap(), rect), 4);
            check("cloudy", bb(|r| cloudy::draw(r, rect).unwrap(), rect), 4);
            check("rain", bb(|r| rain::draw(r, rect).unwrap(), rect), 4);
            check("snow", bb(|r| snow::draw(r, rect).unwrap(), rect), 4);
            check("thunder", bb(|r| thunderstorm::draw(r, rect).unwrap(), rect), 4);
            check("fog", bb(|r| fog::draw(r, rect).unwrap(), rect), 4);
            check("windy", bb(|r| windy::draw(r, rect).unwrap(), rect), 4);
            check("unknown", bb(|r| unknown::draw(r, rect).unwrap(), rect), 4);
            // Sun+cloud compositions are intentionally a touch asymmetric, but
            // must still center reasonably and not spill the square.
            check(
                "partly",
                bb(
                    |r| {
                        sunny::draw_small(r, rect).unwrap();
                        let (cx, cy, size) = anchor(rect);
                        cloudy::draw_at(r, Point::new(cx, cy + size / 12), size).unwrap();
                    },
                    rect,
                ),
                8,
            );
            check("showers", bb(|r| showers::draw(r, rect).unwrap(), rect), 8);
        }
    }
}

/// Weather condition selector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherCondition {
    /// Clear skies.
    Sunny,
    /// Mostly cloudy.
    Cloudy,
    /// Steady rain.
    Rain,
    /// Sun + cloud (variable).
    PartlyCloudy,
    /// Sun + cloud + light precipitation.
    Showers,
    /// Cloud + lightning.
    Thunderstorm,
    /// Cloud + flakes.
    Snow,
    /// Three horizontal lines.
    Fog,
    /// Three lines with hooked ends.
    Windy,
    /// Filled circle with `?`.
    Unknown,
}

/// Compact weather glyph that fills its arranged rect.
pub struct WeatherIcon {
    rect: Rectangle,
    condition: WeatherCondition,
    width: Length,
    height: Length,
}

impl WeatherIcon {
    /// Create a new icon for `condition`. Position and size are
    /// assigned by the parent via `arrange`; use `.width(N).height(N)`
    /// to lock a fixed size.
    pub fn new(condition: WeatherCondition) -> Self {
        Self {
            rect: Rectangle::zero(),
            condition,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<M: Clone> Widget<Rgb565, M> for WeatherIcon {
    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<Rgb565>,
        _theme: &Theme<'t, Rgb565>,
    ) -> Result<(), RenderError> {
        match self.condition {
            WeatherCondition::Sunny => sunny::draw(renderer, self.rect),
            WeatherCondition::Cloudy => cloudy::draw(renderer, self.rect),
            WeatherCondition::Rain => rain::draw(renderer, self.rect),
            WeatherCondition::PartlyCloudy => {
                sunny::draw_small(renderer, self.rect)?;
                // Drop the cloud slightly so it balances the up-left peek sun
                // and the whole glyph centers on the shared anchor.
                let (cx, cy, size) = anchor(self.rect);
                cloudy::draw_at(renderer, Point::new(cx, cy + size / 12), size)
            }
            WeatherCondition::Showers => showers::draw(renderer, self.rect),
            WeatherCondition::Thunderstorm => thunderstorm::draw(renderer, self.rect),
            WeatherCondition::Snow => snow::draw(renderer, self.rect),
            WeatherCondition::Fog => fog::draw(renderer, self.rect),
            WeatherCondition::Windy => windy::draw(renderer, self.rect),
            WeatherCondition::Unknown => unknown::draw(renderer, self.rect),
        }
    }

    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
        let h = self
            .height
            .resolve(constraints.max.height, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, _: Point, _: TouchPhase) -> Option<M> {
        None
    }
}
