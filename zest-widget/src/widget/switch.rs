//! On/off toggle switch: a pill-shaped track with a sliding knob.
//!
//! Immediate-mode: the host owns the `bool` and rebuilds the widget each
//! frame, passing the current state to [`Switch::new`]. A tap emits
//! `on_toggle(!on)` so the host can flip its stored value.
//!
//! Click semantics mirror [`Button`](crate::Button): the press registers
//! on `TouchPhase::Down` and the toggle message fires on `TouchPhase::Up`
//! if the press is still active. The runtime rehydrates the pressed flag
//! each frame via [`mark_pressed`](Widget::mark_pressed).
//!
//! Colors come from the theme's accent [`Component`](zest_theme::Component):
//! the "on" track uses `accent.base`/`accent.pressed`, the "off" track uses
//! `background.divider`, and the knob uses `accent.on_base`.

use super::Widget;
use alloc::boxed::Box;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase, UiAction, WidgetId};
use zest_theme::Theme;

/// Default track width in pixels.
const TRACK_W: u32 = 44;
/// Default track height in pixels.
const TRACK_H: u32 = 24;
/// Inset of the knob from the track edge in pixels.
const KNOB_INSET: i32 = 2;

/// On/off toggle switch. Host owns the `bool`; a tap emits the negated
/// value through [`on_toggle`](Switch::on_toggle).
pub struct Switch<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    on: bool,
    id: Option<WidgetId>,
    on_toggle: Option<Box<dyn Fn(bool) -> M + 'a>>,
    focused: bool,
    pressed: bool,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> Switch<'a, C, M> {
    /// New switch reflecting `on`. Position and size are assigned by the
    /// parent container via `arrange`.
    pub fn new(on: bool) -> Self {
        Self {
            rect: Rectangle::zero(),
            on,
            id: None,
            on_toggle: None,
            focused: false,
            pressed: false,
            width: Length::Fixed(TRACK_W),
            height: Length::Fixed(TRACK_H),
            _color: PhantomData,
        }
    }

    /// Callback invoked with the toggled value (`!on`) on each
    /// tap. Without it the switch is disabled and ignores touches.
    #[must_use]
    pub fn on_toggle<F: Fn(bool) -> M + 'a>(mut self, f: F) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Set a stable id so this switch can participate in focus traversal.
    #[must_use]
    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
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

    /// True iff a toggle callback is bound. Disabled switches render
    /// dimmed and ignore touches.
    pub fn is_enabled(&self) -> bool {
        self.on_toggle.is_some()
    }

    /// The pill track rect, vertically centered within `rect`.
    fn track_rect(&self) -> Rectangle {
        let h = self.rect.size.height.min(TRACK_H);
        let y = self.rect.top_left.y + (self.rect.size.height.saturating_sub(h) / 2) as i32;
        Rectangle::new(
            Point::new(self.rect.top_left.x, y),
            Size::new(self.rect.size.width, h),
        )
    }

    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Switch<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(TRACK_W, constraints.max.width);
        let h = self.height.resolve(TRACK_H, constraints.max.height);
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
        if !self.is_enabled() || !self.hit_test(point) {
            if matches!(phase, TouchPhase::Up | TouchPhase::Moved) {
                self.pressed = false;
            }
            return None;
        }
        match phase {
            TouchPhase::Down => {
                self.pressed = true;
                None
            }
            TouchPhase::Up => {
                if self.pressed {
                    self.pressed = false;
                    self.on_toggle.as_ref().map(|cb| cb(!self.on))
                } else {
                    None
                }
            }
            TouchPhase::Moved => None,
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.is_enabled() && self.hit_test(point) {
            self.pressed = true;
        }
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }

    fn is_focusable(&self) -> bool {
        self.id.is_some() && self.is_enabled()
    }

    fn handle_action(&mut self, action: UiAction) -> Option<M> {
        if !self.is_enabled() {
            return None;
        }

        match action {
            UiAction::Activate => self.on_toggle.as_ref().map(|cb| cb(!self.on)),
            _ => None,
        }
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        self.focused = self.id.is_some() && self.id == focused;
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        if self.is_focusable() && self.hit_test(point) {
            self.id
        } else {
            None
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let track = self.track_rect();
        let accent = &theme.accent;

        let track_color = if !self.is_enabled() {
            theme.background.divider
        } else if self.on {
            if self.pressed {
                accent.pressed
            } else {
                accent.base
            }
        } else {
            theme.background.divider
        };
        renderer.fill_rect(track, track_color)?;
        let border = if self.focused {
            accent.base
        } else {
            accent.border
        };
        renderer.stroke_rect(track, border)?;

        // Knob: a circle inset on whichever end the state selects.
        let radius = (track.size.height as i32 / 2 - KNOB_INSET).max(1) as u32;
        let cy = track.top_left.y + track.size.height as i32 / 2;
        let cx = if self.on {
            track.top_left.x + track.size.width as i32 - KNOB_INSET - radius as i32
        } else {
            track.top_left.x + KNOB_INSET + radius as i32
        };
        let knob_color = if self.is_enabled() {
            accent.on_base
        } else {
            theme.background.base
        };
        renderer.fill_circle(Point::new(cx, cy), radius, knob_color)?;

        Ok(())
    }
}
