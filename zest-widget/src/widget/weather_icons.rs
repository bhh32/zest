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
                cloudy::draw(renderer, self.rect)
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
