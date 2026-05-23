//! Passive rotating-arc loading indicator.
//!
//! The widget itself holds no animation state: it draws a faint full
//! background ring plus a brighter arc segment of a fixed `arc_deg`
//! sweep, rotated to a host-provided `angle` (degrees). The host drives
//! the rotation by storing an angle and bumping it on a timer, then
//! rebuilding the widget each frame — exactly the immediate-mode model
//! used elsewhere in `zest`. See `examples/spinner.rs` for a
//! self-rescheduling `Task::perform` tick loop.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Passive spinner: a rotating arc segment over a faint background ring.
pub struct Spinner<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    /// Current rotation offset in degrees (driven by the host).
    angle: i32,
    /// Length of the visible (bright) arc segment in degrees.
    arc_deg: i32,
    /// Stroke thickness in pixels.
    width: u32,
    /// Whether to draw the faint background ring behind the arc.
    track: bool,
    track_color: Option<C>,
    arc_color: Option<C>,
    w: Length,
    h: Length,
    _phantom: PhantomData<&'a M>,
}

impl<'a, C: PixelColor, M: Clone> Spinner<'a, C, M> {
    /// New spinner rotated to `angle` degrees. Defaults: 90° visible
    /// arc, 5px stroke, background ring on, theme colors, fills its slot.
    pub fn new(angle: i32) -> Self {
        Self {
            rect: Rectangle::zero(),
            angle,
            arc_deg: 90,
            width: 5,
            track: true,
            track_color: None,
            arc_color: None,
            w: Length::Fill,
            h: Length::Fill,
            _phantom: PhantomData,
        }
    }

    /// Builder: set the current rotation angle in degrees.
    #[must_use]
    pub fn angle(mut self, angle: i32) -> Self {
        self.angle = angle;
        self
    }

    /// Builder: length of the bright arc segment in degrees.
    #[must_use]
    pub fn arc_deg(mut self, arc_deg: i32) -> Self {
        self.arc_deg = arc_deg;
        self
    }

    /// Builder: stroke thickness in pixels.
    #[must_use]
    pub fn width_px(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Builder: toggle the faint full background ring.
    #[must_use]
    pub fn track(mut self, on: bool) -> Self {
        self.track = on;
        self
    }

    /// Builder: override the background-ring color (default:
    /// `theme.background.divider`).
    #[must_use]
    pub fn track_color(mut self, color: C) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Builder: override the rotating-arc color (default:
    /// `theme.accent.base`).
    #[must_use]
    pub fn arc_color(mut self, color: C) -> Self {
        self.arc_color = Some(color);
        self
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.w = width.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.h = height.into();
        self
    }

    fn center(&self) -> Point {
        Point::new(
            self.rect.top_left.x + self.rect.size.width as i32 / 2,
            self.rect.top_left.y + self.rect.size.height as i32 / 2,
        )
    }

    fn radius(&self) -> u32 {
        let smaller = self.rect.size.width.min(self.rect.size.height);
        (smaller / 2).saturating_sub(self.width.div_ceil(2))
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Spinner<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.w.resolve(constraints.max.width, constraints.max.width);
        let h = self.h.resolve(constraints.max.height, constraints.max.height);
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
        let arc = self.arc_color.unwrap_or(theme.accent.base);

        if self.track {
            renderer.stroke_arc(center, radius, 0, 360, self.width, track)?;
        }
        renderer.stroke_arc(center, radius, self.angle, self.arc_deg, self.width, arc)?;
        Ok(())
    }
}
