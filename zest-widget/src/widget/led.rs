//! Passive status indicator. Draws a filled circle via
//! [`Renderer::fill_circle`] sized to fit its arranged rect.
//!
//! Two construction styles:
//!
//! * [`LED::new`] — an always-on LED of an explicit color.
//! * [`LED::from_state`] — an on/off LED whose color is chosen from the
//!   `on`/`off` colors (the off color defaults to the theme divider).

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Default LED diameter when sizing intent is `Shrink`.
const DEFAULT_DIAMETER: u32 = 16;

/// A filled-circle status light.
pub struct LED<C: PixelColor, M: Clone> {
    rect: Rectangle,
    /// Color drawn when `on`.
    on_color: C,
    /// Color drawn when not `on`. `None` falls back to the theme divider.
    off_color: Option<C>,
    /// Whether the LED is lit.
    on: bool,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<C: PixelColor, M: Clone> LED<C, M> {
    /// New always-on LED of `color`. Defaults to a fixed square slot;
    /// position is assigned by the parent via `arrange`.
    pub fn new(color: C) -> Self {
        Self {
            rect: Rectangle::zero(),
            on_color: color,
            off_color: None,
            on: true,
            width: Length::Fixed(DEFAULT_DIAMETER),
            height: Length::Fixed(DEFAULT_DIAMETER),
            _phantom: PhantomData,
        }
    }

    /// New on/off LED. `on_color` is drawn when lit; the off color
    /// defaults to `theme.background.divider` unless set via
    /// [`Self::off_color`].
    pub fn from_state(on: bool, on_color: C) -> Self {
        Self {
            rect: Rectangle::zero(),
            on_color,
            off_color: None,
            on,
            width: Length::Fixed(DEFAULT_DIAMETER),
            height: Length::Fixed(DEFAULT_DIAMETER),
            _phantom: PhantomData,
        }
    }

    /// Set the lit state.
    #[must_use]
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Override the lit color.
    #[must_use]
    pub fn on_color(mut self, color: C) -> Self {
        self.on_color = color;
        self
    }

    /// Set the unlit color (default: `theme.background.divider`).
    #[must_use]
    pub fn off_color(mut self, color: C) -> Self {
        self.off_color = Some(color);
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for LED<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(DEFAULT_DIAMETER, constraints.max.width);
        let h = self
            .height
            .resolve(DEFAULT_DIAMETER, constraints.max.height);
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

    fn handle_touch(&mut self, _point: Point, _phase: TouchPhase) -> Option<M> {
        None
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let color = if self.on {
            self.on_color
        } else {
            self.off_color.unwrap_or(theme.background.divider)
        };
        // Largest circle centered in the arranged rect.
        let radius = self.rect.size.width.min(self.rect.size.height) / 2;
        if radius == 0 {
            return Ok(());
        }
        let center = self.rect.top_left
            + Point::new(
                self.rect.size.width as i32 / 2,
                self.rect.size.height as i32 / 2,
            );
        renderer.fill_circle(center, radius, color)
    }
}
