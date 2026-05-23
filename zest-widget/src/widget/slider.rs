//! Horizontal value slider: a track with a filled portion and a draggable
//! knob.
//!
//! Immediate-mode: the host owns the `f32` value and rebuilds the widget
//! each frame, passing the current value to [`Slider::new`]. The value is
//! computed *directly* from the touch x position within the track on both
//! `TouchPhase::Down` and `TouchPhase::Moved`, so dragging works without
//! needing a remembered drag origin. The new value is clamped to the
//! configured range and emitted via [`on_change`](Slider::on_change).
//!
//! Colors come from the theme's accent [`Component`](zest_theme::Component):
//! the filled portion uses `accent.base`, the unfilled track uses
//! `background.divider`, and the knob uses `accent.on_base` with an
//! `accent.border` outline.

use super::Widget;
use alloc::boxed::Box;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Track thickness in pixels.
const TRACK_THICKNESS: u32 = 6;
/// Knob radius in pixels.
const KNOB_RADIUS: u32 = 9;
/// Default intrinsic width when sized `Shrink`.
const INTRINSIC_W: u32 = 160;

/// Horizontal value slider. Host owns the value; dragging emits the new
/// clamped value through [`on_change`](Slider::on_change).
pub struct Slider<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    value: f32,
    min: f32,
    max: f32,
    on_change: Option<Box<dyn Fn(f32) -> M + 'a>>,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> Slider<'a, C, M> {
    /// New slider showing `value`. Default range is `0.0..=1.0`. Position
    /// and size are assigned by the parent container via `arrange`.
    pub fn new(value: f32) -> Self {
        Self {
            rect: Rectangle::zero(),
            value,
            min: 0.0,
            max: 1.0,
            on_change: None,
            width: Length::Fill,
            height: Length::Fixed(2 * KNOB_RADIUS),
            _color: PhantomData,
        }
    }

    /// Builder: inclusive value range. If `min >= max` the range collapses
    /// and the slider reports `min`.
    #[must_use]
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Builder: callback invoked with the new clamped value whenever the
    /// knob is pressed or dragged. Without it the slider is disabled and
    /// ignores touches.
    #[must_use]
    pub fn on_change<F: Fn(f32) -> M + 'a>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
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

    /// True iff a change callback is bound. Disabled sliders render dimmed
    /// and ignore touches.
    pub fn is_enabled(&self) -> bool {
        self.on_change.is_some()
    }

    /// Fraction (0.0..=1.0) of the current value within the range.
    fn fraction(&self) -> f32 {
        if self.max <= self.min {
            0.0
        } else {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }

    /// Usable span for the knob center: leaves a `KNOB_RADIUS` margin at
    /// each end so the knob stays inside the widget.
    fn span(&self) -> (i32, i32) {
        let left = self.rect.top_left.x + KNOB_RADIUS as i32;
        let right = self.rect.top_left.x + self.rect.size.width as i32 - KNOB_RADIUS as i32;
        (left, right.max(left))
    }

    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }

    /// Convert a touch x into a clamped value within the range.
    fn value_at(&self, x: i32) -> f32 {
        let (left, right) = self.span();
        let frac = if right <= left {
            0.0
        } else {
            ((x - left) as f32 / (right - left) as f32).clamp(0.0, 1.0)
        };
        let v = self.min + frac * (self.max - self.min);
        v.clamp(self.min.min(self.max), self.min.max(self.max))
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Slider<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(INTRINSIC_W, constraints.max.width);
        let h = self.height.resolve(2 * KNOB_RADIUS, constraints.max.height);
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

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        let Some(cb) = self.on_change.as_ref() else {
            return None;
        };
        match phase {
            // Drive value directly from x on press and during drag.
            TouchPhase::Down => {
                if self.hit_test(point) {
                    Some(cb(self.value_at(point.x)))
                } else {
                    None
                }
            }
            TouchPhase::Moved => {
                // Only the slider currently under the finger responds. In
                // zest's transient-widget model there is no per-widget
                // "I own this drag" state, so without this hit-test every
                // move event would be greedily consumed by whichever slider
                // the container reaches first — dragging one slider would
                // move another. Each slider occupies its own row, so a
                // horizontal drag stays within bounds.
                if self.hit_test(point) {
                    Some(cb(self.value_at(point.x)))
                } else {
                    None
                }
            }
            TouchPhase::Up => None,
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let accent = &theme.accent;
        let (left, right) = self.span();
        let cy = self.rect.top_left.y + self.rect.size.height as i32 / 2;
        let track_top = cy - TRACK_THICKNESS as i32 / 2;

        let track_color = theme.background.divider;
        let fill_color = if self.is_enabled() {
            accent.base
        } else {
            theme.background.divider
        };

        // Full track.
        let track = Rectangle::new(
            Point::new(left, track_top),
            Size::new((right - left).max(0) as u32, TRACK_THICKNESS),
        );
        renderer.fill_rect(track, track_color)?;

        // Knob center x from the current fraction.
        let knob_x = left + ((right - left) as f32 * self.fraction()) as i32;

        // Filled portion from the left up to the knob.
        let filled = Rectangle::new(
            Point::new(left, track_top),
            Size::new((knob_x - left).max(0) as u32, TRACK_THICKNESS),
        );
        renderer.fill_rect(filled, fill_color)?;

        // Knob: a border ring (full radius) with the knob fill punched on
        // top, leaving a 2px accent outline — the renderer has no
        // stroke_circle primitive.
        let knob_color = if self.is_enabled() {
            accent.on_base
        } else {
            theme.background.base
        };
        let center = Point::new(knob_x, cy);
        renderer.fill_circle(center, KNOB_RADIUS, accent.border)?;
        renderer.fill_circle(center, KNOB_RADIUS.saturating_sub(2), knob_color)?;

        Ok(())
    }
}
