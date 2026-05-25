//! Passive scale: evenly spaced tick marks with optional numeric labels.
//!
//! Two modes:
//! - [`ScaleMode::Linear`] draws a ruler: a baseline with major/minor
//!   ticks running left-to-right, labels beneath the major ticks.
//! - [`ScaleMode::Circular`] draws a gauge: ticks radiating inward from
//!   an arc, with labels just inside the major ticks. The arc itself is
//!   drawn via [`Renderer::stroke_arc`].
//!
//! The host supplies the value range, the tick counts, and (optionally)
//! whether to label major ticks with their numeric value. The widget is
//! non-interactive. Pair it with [`Arc`](super::arc::Arc) to build a
//! labelled gauge.

use super::Widget;
use alloc::{format, string::String};
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase, arc_sin_cos};
use zest_theme::Theme;

/// Layout mode for a [`Scale`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    /// Horizontal ruler: ticks along a baseline, labels beneath.
    Linear,
    /// Circular gauge: ticks radiating inward from an arc.
    Circular,
}

/// Passive tick-mark scale (ruler or gauge).
pub struct Scale<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    mode: ScaleMode,
    min: f32,
    max: f32,
    /// Number of major divisions (so `major_ticks + 1` major tick marks).
    major_ticks: u32,
    /// Minor ticks drawn between each pair of major ticks.
    minor_per_major: u32,
    /// Show numeric labels at major ticks.
    labels: bool,
    /// Circular-mode geometry (ignored in linear mode).
    start_deg: i32,
    sweep_deg: i32,
    color: Option<C>,
    label_color: Option<C>,
    font: Option<&'a MonoFont<'a>>,
    w: Length,
    h: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor, M: Clone> Scale<'a, C, M> {
    /// New scale over `min..=max`. Defaults: linear, 5 major divisions,
    /// 4 minor ticks between majors, labels on, theme colors.
    pub fn new(min: f32, max: f32) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            rect: Rectangle::zero(),
            mode: ScaleMode::Linear,
            min,
            max,
            major_ticks: 5,
            minor_per_major: 4,
            labels: true,
            start_deg: 225,
            sweep_deg: -270,
            color: None,
            label_color: None,
            font: None,
            w: Length::Fill,
            h: Length::Fill,
            _phantom: PhantomData,
        }
    }

    /// Layout mode (linear ruler or circular gauge).
    #[must_use]
    pub fn mode(mut self, mode: ScaleMode) -> Self {
        self.mode = mode;
        self
    }

    /// Number of major divisions (`n` divisions ⇒ `n + 1`
    /// major ticks). Clamped to at least 1.
    #[must_use]
    pub fn major_ticks(mut self, n: u32) -> Self {
        self.major_ticks = n.max(1);
        self
    }

    /// Minor ticks drawn between each pair of major ticks.
    #[must_use]
    pub fn minor_per_major(mut self, n: u32) -> Self {
        self.minor_per_major = n;
        self
    }

    /// Toggle numeric labels at major ticks.
    #[must_use]
    pub fn labels(mut self, on: bool) -> Self {
        self.labels = on;
        self
    }

    /// Starting angle for circular mode (0° points right).
    #[must_use]
    pub fn start_deg(mut self, start_deg: i32) -> Self {
        self.start_deg = start_deg;
        self
    }

    /// Total sweep for circular mode in degrees.
    #[must_use]
    pub fn sweep_deg(mut self, sweep_deg: i32) -> Self {
        self.sweep_deg = sweep_deg;
        self
    }

    /// Override tick/baseline color (default:
    /// `theme.background.on_base`).
    #[must_use]
    pub fn color(mut self, color: C) -> Self {
        self.color = Some(color);
        self
    }

    /// Override label color (default: `theme.palette.neutral_2`).
    #[must_use]
    pub fn label_color(mut self, color: C) -> Self {
        self.label_color = Some(color);
        self
    }

    /// Override label font (default: `theme.typography.caption`).
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font = Some(font);
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

    /// Value at major-tick index `i` (`0..=major_ticks`).
    fn value_at(&self, i: u32) -> f32 {
        let t = i as f32 / self.major_ticks as f32;
        self.min + (self.max - self.min) * t
    }

    /// Format a numeric tick value compactly (drops a trailing `.0`).
    fn label_for(v: f32) -> String {
        if (v - libm_round(v)).abs() < 0.05 {
            format!("{}", libm_round(v) as i64)
        } else {
            format!("{v:.1}")
        }
    }

    /// Total tick count between (and including) the two ends.
    fn total_ticks(&self) -> u32 {
        self.major_ticks * (self.minor_per_major + 1)
    }
}

/// Round-half-to-even substitute that avoids `std`/`libm`: round to the
/// nearest integer toward the closer side.
fn libm_round(v: f32) -> f32 {
    if v >= 0.0 {
        (v + 0.5) as i64 as f32
    } else {
        (v - 0.5) as i64 as f32
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Scale<'a, C, M> {
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
        let tick = self.color.unwrap_or(theme.background.on_base);
        let label_color = self.label_color.unwrap_or(theme.palette.neutral_2);
        let font = self.font.unwrap_or(theme.typography.caption);

        match self.mode {
            ScaleMode::Linear => self.draw_linear(renderer, tick, label_color, font),
            ScaleMode::Circular => self.draw_circular(renderer, tick, label_color, font),
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Scale<'a, C, M> {
    fn draw_linear(
        &self,
        renderer: &mut dyn Renderer<C>,
        tick: C,
        label_color: C,
        font: &MonoFont<'_>,
    ) -> Result<(), RenderError> {
        let r = self.rect;
        if r.size.width == 0 || r.size.height == 0 {
            return Ok(());
        }
        let major_len = (r.size.height / 3).max(4) as i32;
        let minor_len = (major_len / 2).max(2);
        // Baseline near the top so labels have room beneath the ruler.
        let baseline_y = r.top_left.y + 1;
        let left = r.top_left.x;
        let width = r.size.width.saturating_sub(1) as i32;

        // Baseline.
        renderer.stroke_line(
            Point::new(left, baseline_y),
            Point::new(left + width, baseline_y),
            tick,
            1,
        )?;

        let total = self.total_ticks();
        for i in 0..=total {
            let is_major = i % (self.minor_per_major + 1) == 0;
            let x = left + (width * i as i32) / total as i32;
            let len = if is_major { major_len } else { minor_len };
            renderer.stroke_line(
                Point::new(x, baseline_y),
                Point::new(x, baseline_y + len),
                tick,
                1,
            )?;
            if is_major && self.labels {
                let major_index = i / (self.minor_per_major + 1);
                let text = Self::label_for(self.value_at(major_index));
                let label_y = baseline_y + major_len + 2 + font.character_size.height as i32;
                renderer.draw_text(
                    &text,
                    Point::new(x, label_y),
                    font,
                    label_color,
                    Alignment::Center,
                )?;
            }
        }
        Ok(())
    }

    fn draw_circular(
        &self,
        renderer: &mut dyn Renderer<C>,
        tick: C,
        label_color: C,
        font: &MonoFont<'_>,
    ) -> Result<(), RenderError> {
        let r = self.rect;
        let center = Point::new(
            r.top_left.x + r.size.width as i32 / 2,
            r.top_left.y + r.size.height as i32 / 2,
        );
        let smaller = r.size.width.min(r.size.height);
        let outer = (smaller / 2).saturating_sub(2);
        if outer == 0 {
            return Ok(());
        }
        let major_len = (outer / 6).max(4);
        let minor_len = (major_len / 2).max(2);

        // The reference arc the ticks hang from.
        renderer.stroke_arc(center, outer, self.start_deg, self.sweep_deg, 2, tick)?;

        let total = self.total_ticks();
        let outer_r = outer as f32;
        // Labels sit inside the major ticks.
        let label_r = (outer as f32) - major_len as f32 - font.character_size.height as f32;

        for i in 0..=total {
            let is_major = i % (self.minor_per_major + 1) == 0;
            let frac = i as f32 / total as f32;
            let deg = self.start_deg + (self.sweep_deg as f32 * frac) as i32;
            let (s, c) = arc_sin_cos(deg);
            let inner_r = outer_r - if is_major { major_len } else { minor_len } as f32;

            let p_outer = Point::new(
                center.x + (c * outer_r) as i32,
                center.y - (s * outer_r) as i32,
            );
            let p_inner = Point::new(
                center.x + (c * inner_r) as i32,
                center.y - (s * inner_r) as i32,
            );
            renderer.stroke_line(p_outer, p_inner, tick, 1)?;

            if is_major && self.labels && label_r > 0.0 {
                let major_index = i / (self.minor_per_major + 1);
                let text = Self::label_for(self.value_at(major_index));
                let lp = Point::new(
                    center.x + (c * label_r) as i32,
                    // Nudge the baseline so the glyph centers vertically.
                    center.y - (s * label_r) as i32 + font.character_size.height as i32 / 3,
                );
                renderer.draw_text(&text, lp, font, label_color, Alignment::Center)?;
            }
        }
        Ok(())
    }
}
