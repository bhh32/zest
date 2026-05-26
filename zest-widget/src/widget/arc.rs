//! Passive arc gauge: a background track arc with a value arc drawn on
//! top, sweeping from a start angle toward an end angle in proportion to
//! a value within `min..=max`.
//!
//! The center and radius are derived from the arranged rectangle: the arc
//! is centered in the rect and the radius is half the smaller dimension
//! (inset by the stroke width so the stroke stays inside the bounds).
//! Angles follow the [`Renderer::stroke_arc`] convention — 0° points
//! right and a positive sweep travels counter-clockwise (toward the
//! top). The widget is non-interactive.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Passive value arc with a background track.
pub struct Arc<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    /// Current value, clamped to `min..=max` at draw time.
    value: f32,
    min: f32,
    max: f32,
    /// Angle (degrees) at which the track/value arc begins.
    start_deg: i32,
    /// Total sweep (degrees) the full range maps to. Positive sweeps
    /// counter-clockwise; negative sweeps clockwise.
    sweep_deg: i32,
    /// Stroke thickness of both arcs, in pixels.
    width: u32,
    track_color: Option<C>,
    value_color: Option<C>,
    w: Length,
    h: Length,
    _phantom: PhantomData<&'a M>,
}

impl<'a, C: PixelColor, M: Clone> Arc<'a, C, M> {
    /// New arc displaying `value` within `min..=max`. Defaults: a 270°
    /// sweep starting at 225° (a bottom-gap gauge), 6px stroke, fills
    /// its slot. Theme colors are used unless overridden.
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            rect: Rectangle::zero(),
            value,
            min,
            max,
            start_deg: 225,
            sweep_deg: -270,
            width: 6,
            track_color: None,
            value_color: None,
            w: Length::Fill,
            h: Length::Fill,
            _phantom: PhantomData,
        }
    }

    /// Set the value (clamped to `min..=max` at draw time).
    #[must_use]
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// The value range. Reordered so `min <= max`.
    #[must_use]
    pub fn range(mut self, min: f32, max: f32) -> Self {
        if min <= max {
            self.min = min;
            self.max = max;
        } else {
            self.min = max;
            self.max = min;
        }
        self
    }

    /// Starting angle in degrees (0° points right).
    #[must_use]
    pub fn start_deg(mut self, start_deg: i32) -> Self {
        self.start_deg = start_deg;
        self
    }

    /// Total sweep in degrees that the full range spans.
    /// Positive sweeps counter-clockwise; negative clockwise.
    #[must_use]
    pub fn sweep_deg(mut self, sweep_deg: i32) -> Self {
        self.sweep_deg = sweep_deg;
        self
    }

    /// Stroke thickness of both arcs in pixels.
    #[must_use]
    pub fn width_px(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Override the background-track color (default:
    /// `theme.background.divider`).
    #[must_use]
    pub fn track_color(mut self, color: C) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Override the value-arc color (default: `theme.accent.base`).
    #[must_use]
    pub fn value_color(mut self, color: C) -> Self {
        self.value_color = Some(color);
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.w = width.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.h = height.into();
        self
    }

    /// Fraction of the range the current value represents, in `0.0..=1.0`.
    fn fraction(&self) -> f32 {
        let span = self.max - self.min;
        if span <= 0.0 {
            return 0.0;
        }
        ((self.value - self.min) / span).clamp(0.0, 1.0)
    }

    /// Center point of the arc within the arranged rect.
    fn center(&self) -> Point {
        Point::new(
            self.rect.top_left.x + self.rect.size.width as i32 / 2,
            self.rect.top_left.y + self.rect.size.height as i32 / 2,
        )
    }

    /// Radius derived from the rect, inset so the stroke stays inside.
    fn radius(&self) -> u32 {
        let smaller = self.rect.size.width.min(self.rect.size.height);
        (smaller / 2).saturating_sub(self.width.div_ceil(2))
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Arc<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.w.resolve(constraints.max.width, constraints.max.width);
        let h = self
            .h
            .resolve(constraints.max.height, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.w, self.h)
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
        let center = self.center();
        let radius = self.radius();
        if radius == 0 {
            return Ok(());
        }

        let track = self.track_color.unwrap_or(theme.background.divider);
        let value = self.value_color.unwrap_or(theme.accent.base);

        // Background track spans the full range.
        renderer.stroke_arc(
            center,
            radius,
            self.start_deg,
            self.sweep_deg,
            self.width,
            track,
        )?;

        // Value arc spans a fraction of the sweep.
        let value_sweep = (self.sweep_deg as f32 * self.fraction()) as i32;
        if value_sweep != 0 {
            renderer.stroke_arc(
                center,
                radius,
                self.start_deg,
                value_sweep,
                self.width,
                value,
            )?;
        }

        Ok(())
    }
}
