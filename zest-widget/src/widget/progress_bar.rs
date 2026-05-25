//! Passive progress bar: a track with a filled portion reflecting a value.
//!
//! Non-interactive — ignores touch entirely. The host owns the value and
//! rebuilds the widget each frame. The fraction shown is `(value - min) /
//! (max - min)`, clamped to `0.0..=1.0`. The default range is `0.0..=1.0`,
//! so `ProgressBar::new(0.5)` is half full.
//!
//! Colors default to the theme: the filled portion uses `accent.base`
//! (override with [`color`](ProgressBar::color)) and the track uses
//! `background.divider`.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Default bar height in pixels.
const BAR_H: u32 = 10;
/// Default intrinsic width when sized `Shrink`.
const INTRINSIC_W: u32 = 160;

/// Passive progress bar reflecting a value within a range.
pub struct ProgressBar<C: PixelColor, M: Clone> {
    rect: Rectangle,
    value: f32,
    min: f32,
    max: f32,
    color: Option<C>,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<C: PixelColor, M: Clone> ProgressBar<C, M> {
    /// New progress bar showing `value`. Default range is `0.0..=1.0`.
    pub fn new(value: f32) -> Self {
        Self {
            rect: Rectangle::zero(),
            value,
            min: 0.0,
            max: 1.0,
            color: None,
            width: Length::Fill,
            height: Length::Fixed(BAR_H),
            _phantom: PhantomData,
        }
    }

    /// Inclusive value range mapped to the fill fraction.
    #[must_use]
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Explicit fill color (default: `theme.accent.base`).
    #[must_use]
    pub fn color(mut self, color: C) -> Self {
        self.color = Some(color);
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

    /// Fraction (0.0..=1.0) of the value within the range.
    fn fraction(&self) -> f32 {
        if self.max <= self.min {
            0.0
        } else {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for ProgressBar<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(INTRINSIC_W, constraints.max.width);
        let h = self.height.resolve(BAR_H, constraints.max.height);
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
        // Track.
        renderer.fill_rect(self.rect, theme.background.divider)?;

        // Filled portion.
        let fill_w = (self.rect.size.width as f32 * self.fraction()) as u32;
        if fill_w > 0 {
            let fill_color = self.color.unwrap_or(theme.accent.base);
            let filled =
                Rectangle::new(self.rect.top_left, Size::new(fill_w, self.rect.size.height));
            renderer.fill_rect(filled, fill_color)?;
        }

        Ok(())
    }
}
