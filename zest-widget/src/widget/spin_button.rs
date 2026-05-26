//! Numeric stepper with `-` / `+` buttons flanking a value label.
//!
//! Stateless w.r.t. the value: the host owns the `i32` and rebuilds the
//! widget each frame. The `on_change` callback receives the new value
//! (clamped to `min..=max`) when a side is tapped.

use super::Widget;
use alloc::{boxed::Box, format, string::String};
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase, UiAction, WidgetId};
use zest_theme::{ButtonCatalog, ButtonClass, Status, Theme};

const H_BUTTON_W: u32 = 28;
const V_BUTTON_H: u32 = 24;

/// Layout direction for a [`SpinButton`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpinOrientation {
    /// `[-] [value] [+]`
    Horizontal,
    /// `[+]` / `[value]` / `[-]` (plus on top).
    Vertical,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Side {
    Plus,
    Minus,
}

/// Numeric stepper widget.
pub struct SpinButton<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    id: Option<WidgetId>,
    value: i32,
    min: i32,
    max: i32,
    step: i32,
    orientation: SpinOrientation,
    display: Option<String>,
    on_change: Option<Box<dyn Fn(i32) -> M + 'a>>,
    width: Length,
    height: Length,
    focused: Option<Side>,
    pressed: Option<Side>,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> SpinButton<'a, C, M> {
    /// New spinner showing `value`. Defaults: range `0..=99`, step 1,
    /// horizontal.
    pub fn new(value: i32) -> Self {
        Self {
            rect: Rectangle::zero(),
            id: None,
            value,
            min: 0,
            max: 99,
            step: 1,
            orientation: SpinOrientation::Horizontal,
            display: None,
            on_change: None,
            width: Length::Fill,
            height: Length::Fill,
            focused: None,
            pressed: None,
            _color: PhantomData,
        }
    }

    /// Minimum value (inclusive).
    #[must_use]
    pub fn min(mut self, min: i32) -> Self {
        self.min = min;
        self
    }

    /// Maximum value (inclusive).
    #[must_use]
    pub fn max(mut self, max: i32) -> Self {
        self.max = max;
        self
    }

    /// Step applied on each `+`/`-` tap.
    #[must_use]
    pub fn step(mut self, step: i32) -> Self {
        self.step = step;
        self
    }

    /// Layout orientation.
    #[must_use]
    pub fn orientation(mut self, o: SpinOrientation) -> Self {
        self.orientation = o;
        self
    }

    /// Override the displayed string. Default is `format!("{value}")`.
    #[must_use]
    pub fn display(mut self, s: impl Into<String>) -> Self {
        self.display = Some(s.into());
        self
    }

    /// Set a stable base id so both sides can participate in focus traversal.
    #[must_use]
    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    /// Callback fired with the clamped new value on each tap.
    #[must_use]
    pub fn on_change<F: Fn(i32) -> M + 'a>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    fn intrinsic(&self) -> Size {
        match self.orientation {
            SpinOrientation::Horizontal => Size::new(90, 24),
            SpinOrientation::Vertical => Size::new(28, 80),
        }
    }

    fn minus_rect(&self) -> Rectangle {
        let r = self.rect;
        match self.orientation {
            SpinOrientation::Horizontal => {
                Rectangle::new(r.top_left, Size::new(H_BUTTON_W, r.size.height))
            }
            SpinOrientation::Vertical => Rectangle::new(
                r.top_left + Point::new(0, r.size.height.saturating_sub(V_BUTTON_H) as i32),
                Size::new(r.size.width, V_BUTTON_H),
            ),
        }
    }

    fn plus_rect(&self) -> Rectangle {
        let r = self.rect;
        match self.orientation {
            SpinOrientation::Horizontal => Rectangle::new(
                r.top_left + Point::new(r.size.width.saturating_sub(H_BUTTON_W) as i32, 0),
                Size::new(H_BUTTON_W, r.size.height),
            ),
            SpinOrientation::Vertical => {
                Rectangle::new(r.top_left, Size::new(r.size.width, V_BUTTON_H))
            }
        }
    }

    fn value_rect(&self) -> Rectangle {
        let r = self.rect;
        match self.orientation {
            SpinOrientation::Horizontal => {
                let w = r.size.width.saturating_sub(H_BUTTON_W * 2);
                Rectangle::new(
                    r.top_left + Point::new(H_BUTTON_W as i32, 0),
                    Size::new(w, r.size.height),
                )
            }
            SpinOrientation::Vertical => {
                let h = r.size.height.saturating_sub(V_BUTTON_H * 2);
                Rectangle::new(
                    r.top_left + Point::new(0, V_BUTTON_H as i32),
                    Size::new(r.size.width, h),
                )
            }
        }
    }

    fn hit_test(&self, point: Point) -> Option<Side> {
        if rect_contains(self.minus_rect(), point) {
            Some(Side::Minus)
        } else if rect_contains(self.plus_rect(), point) {
            Some(Side::Plus)
        } else {
            None
        }
    }

    fn side_enabled(&self, side: Side) -> bool {
        if self.on_change.is_none() {
            return false;
        }
        match side {
            Side::Minus => self.value > self.min,
            Side::Plus => self.value < self.max,
        }
    }

    fn side_status(&self, side: Side) -> Status {
        if !self.side_enabled(side) {
            Status::Disabled
        } else if self.pressed == Some(side) {
            Status::Pressed
        } else if self.focused == Some(side) {
            Status::Focused
        } else {
            Status::Active
        }
    }

    fn apply(&self, side: Side) -> i32 {
        let next = match side {
            Side::Minus => self.value.saturating_sub(self.step),
            Side::Plus => self.value.saturating_add(self.step),
        };
        next.clamp(self.min, self.max)
    }

    fn side_id(&self, side: Side) -> Option<WidgetId> {
        self.id.map(|base| {
            let offset = match side {
                Side::Minus => 1,
                Side::Plus => 2,
            };
            WidgetId::new(base.raw().wrapping_add(offset))
        })
    }

    fn focused_side(&self, target: WidgetId) -> Option<Side> {
        [Side::Minus, Side::Plus]
            .into_iter()
            .find(|side| self.side_id(*side) == Some(target))
    }

    fn ordered_sides(&self) -> [Side; 2] {
        match self.orientation {
            SpinOrientation::Horizontal => [Side::Minus, Side::Plus],
            SpinOrientation::Vertical => [Side::Plus, Side::Minus],
        }
    }

    fn emit_change(&self, side: Side) -> Option<M> {
        if !self.side_enabled(side) {
            return None;
        }

        self.on_change.as_ref().map(|cb| cb(self.apply(side)))
    }
}

fn rect_contains(rect: Rectangle, p: Point) -> bool {
    let tl = rect.top_left;
    let br = tl + Point::new(rect.size.width as i32, rect.size.height as i32);
    p.x >= tl.x && p.x < br.x && p.y >= tl.y && p.y < br.y
}

/// Horizontal spinner `[-] [value] [+]`.
pub fn horizontal_spin_button<'a, C: PixelColor, M: Clone>(value: i32) -> SpinButton<'a, C, M> {
    SpinButton::new(value).orientation(SpinOrientation::Horizontal)
}

/// Vertical spinner stacked plus-on-top / value / minus-on-bottom.
pub fn vertical_spin_button<'a, C: PixelColor, M: Clone>(value: i32) -> SpinButton<'a, C, M> {
    SpinButton::new(value).orientation(SpinOrientation::Vertical)
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for SpinButton<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let intrinsic = self.intrinsic();
        let w = self.width.resolve(intrinsic.width, constraints.max.width);
        let h = self
            .height
            .resolve(intrinsic.height, constraints.max.height);
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
        if self.on_change.is_none() {
            return None;
        }
        match phase {
            TouchPhase::Down => {
                let hit = self.hit_test(point);
                self.pressed = hit.filter(|s| self.side_enabled(*s));
                None
            }
            TouchPhase::Moved => {
                if self.pressed.is_some() && self.hit_test(point) != self.pressed {
                    self.pressed = None;
                }
                None
            }
            TouchPhase::Up => {
                let now = self.hit_test(point);
                let was = self.pressed.take();
                if let (Some(n), Some(w)) = (now, was) {
                    if n == w {
                        if let Some(cb) = self.on_change.as_ref() {
                            return Some(cb(self.apply(n)));
                        }
                    }
                }
                None
            }
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.pressed.is_none() && self.on_change.is_some() {
            if let Some(side) = self.hit_test(point) {
                if self.side_enabled(side) {
                    self.pressed = Some(side);
                }
            }
        }
    }

    fn collect_focusable(&self, out: &mut alloc::vec::Vec<WidgetId>) {
        for side in self.ordered_sides() {
            if self.side_enabled(side)
                && let Some(id) = self.side_id(side)
            {
                out.push(id);
            }
        }
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        self.focused = focused.and_then(|target| self.focused_side(target));
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        let focused_side = self.focused_side(target)?;
        match action {
            UiAction::Activate => self.emit_change(focused_side),
            UiAction::Increment | UiAction::NavigateRight | UiAction::NavigateUp => {
                self.emit_change(Side::Plus)
            }
            UiAction::Decrement | UiAction::NavigateLeft | UiAction::NavigateDown => {
                self.emit_change(Side::Minus)
            }
            _ => None,
        }
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        let side = self.focused_side(target)?;
        Some(match side {
            Side::Minus => self.minus_rect(),
            Side::Plus => self.plus_rect(),
        })
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        self.hit_test(point).and_then(|side| self.side_id(side))
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let minus_rect = self.minus_rect();
        let plus_rect = self.plus_rect();
        let value_rect = self.value_rect();

        let minus = theme.button(ButtonClass::Standard, self.side_status(Side::Minus));
        let plus = theme.button(ButtonClass::Standard, self.side_status(Side::Plus));

        if let Some(bg) = minus.background {
            renderer.fill_rect(minus_rect, bg)?;
        }
        if let Some(border) = minus.border {
            renderer.stroke_rect(minus_rect, border)?;
        }
        if let Some(bg) = plus.background {
            renderer.fill_rect(plus_rect, bg)?;
        }
        if let Some(border) = plus.border {
            renderer.stroke_rect(plus_rect, border)?;
        }

        renderer.fill_rect(value_rect, theme.background.base)?;

        let body = theme.typography.body;
        let glyph_y = |rect: Rectangle| {
            rect.top_left.y
                + (rect.size.height / 2) as i32
                + (body.character_size.height / 3) as i32
        };

        renderer.draw_text(
            "-",
            Point::new(
                minus_rect.top_left.x + (minus_rect.size.width / 2) as i32,
                glyph_y(minus_rect),
            ),
            body,
            minus.text,
            Alignment::Center,
        )?;
        renderer.draw_text(
            "+",
            Point::new(
                plus_rect.top_left.x + (plus_rect.size.width / 2) as i32,
                glyph_y(plus_rect),
            ),
            body,
            plus.text,
            Alignment::Center,
        )?;

        let label_owned;
        let label: &str = match self.display.as_deref() {
            Some(s) => s,
            None => {
                label_owned = format!("{}", self.value);
                &label_owned
            }
        };
        renderer.draw_text(
            label,
            Point::new(
                value_rect.top_left.x + (value_rect.size.width / 2) as i32,
                glyph_y(value_rect),
            ),
            body,
            theme.background.on_base,
            Alignment::Center,
        )?;

        Ok(())
    }
}
