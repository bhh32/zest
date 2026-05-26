//! Tappable rectangle with a label that emits a Message on release.
//!
//! Click semantics: press registers on `TouchPhase::Down` (sets
//! `pressed = true`, no message), message fires on `TouchPhase::Up` if
//! `pressed` is still set. The runtime rehydrates `pressed` each frame
//! from `pressed_at: Option<Point>` via [`mark_pressed`](Widget::mark_pressed),
//! so the press persists visually across the Down/Up gesture and
//! drag-off-to-cancel works.
//!
//! Styling is resolved through the theme's [`ButtonCatalog`] using the
//! widget's [`ButtonClass`] (variant) and current [`Status`] (state).
//! `.class(ButtonClass::Suggested)` for primary actions,
//! `.class(ButtonClass::Destructive)` for delete/cancel, etc.

use super::Widget;
use alloc::string::String;
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase, UiAction, WidgetId};
use zest_theme::{ButtonCatalog, ButtonClass, Status, Theme};

/// Tappable button with a label.
pub struct Button<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    label: String,
    on_press: Option<M>,
    id: Option<WidgetId>,
    focused: bool,
    pressed: bool,
    class: ButtonClass,
    font_override: Option<&'a MonoFont<'a>>,
    intrinsic_size: Size,
    width: Length,
    height: Length,
    // C only appears in trait bounds (Theme<C>, dyn Renderer<C>) — pin
    // it to a field so the struct's type parameter has a referent.
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> Button<'a, C, M> {
    /// Create a new button with a label. Defaults to
    /// [`ButtonClass::Standard`]. Position and size are assigned by
    /// the parent container via `arrange`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            rect: Rectangle::zero(),
            label: label.into(),
            on_press: None,
            id: None,
            focused: false,
            pressed: false,
            class: ButtonClass::Standard,
            font_override: None,
            intrinsic_size: Size::new(48, 32),
            width: Length::Fill,
            height: Length::Fill,
            _color: PhantomData,
        }
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

    /// Set the message emitted on release.
    #[must_use]
    pub fn on_press(mut self, msg: M) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Conditionally set the message. `None` leaves the button
    /// disabled.
    #[must_use]
    pub fn on_press_maybe(mut self, msg: Option<M>) -> Self {
        self.on_press = msg;
        self
    }

    /// Set a stable id so this button can participate in focus traversal.
    #[must_use]
    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    /// Replace the label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Select the semantic [`ButtonClass`] (variant). Default
    /// is `Standard`. Use `Suggested` for primary actions,
    /// `Destructive` for delete/cancel, `Text` for tertiary.
    #[must_use]
    pub fn class(mut self, class: ButtonClass) -> Self {
        self.class = class;
        self
    }

    /// Override the default font for this instance.
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font_override = Some(font);
        self
    }

    /// Intrinsic size used by `measure` when constraints are
    /// loose.
    #[must_use]
    pub fn intrinsic_size(mut self, size: Size) -> Self {
        self.intrinsic_size = size;
        self
    }

    /// True iff this button has a message bound. Disabled buttons
    /// render dimmed and ignore touches.
    pub fn is_enabled(&self) -> bool {
        self.on_press.is_some()
    }

    fn status(&self) -> Status {
        if !self.is_enabled() {
            Status::Disabled
        } else if self.pressed {
            Status::Pressed
        } else if self.focused {
            Status::Focused
        } else {
            Status::Active
        }
    }

    fn resolved_font<'t>(&'t self, theme: &'t Theme<'a, C>) -> &'t MonoFont<'a> {
        self.font_override.unwrap_or(theme.default_font())
    }

    fn hit_test(&self, point: Point) -> bool {
        let top_left = self.rect.top_left;
        let bot_right =
            top_left + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= top_left.x
            && point.x < bot_right.x
            && point.y >= top_left.y
            && point.y < bot_right.y
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Button<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(self.intrinsic_size.width, constraints.max.width);
        let h = self
            .height
            .resolve(self.intrinsic_size.height, constraints.max.height);
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
        if !self.is_enabled() {
            return None;
        }
        if !self.hit_test(point) {
            return None;
        }
        match phase {
            TouchPhase::Down => {
                self.pressed = true;
                None
            }
            TouchPhase::Up => {
                if self.pressed {
                    self.on_press.clone()
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
            UiAction::Activate => self.on_press.clone(),
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
        let appearance = theme.button(self.class, self.status());
        let font = self.resolved_font(theme);

        if let Some(bg) = appearance.background {
            renderer.fill_rect(self.rect, bg)?;
        }
        if let Some(border) = appearance.border {
            renderer.stroke_rect(self.rect, border)?;
        }

        let center_x = self.rect.top_left.x + self.rect.size.width as i32 / 2;
        let center_y = self.rect.top_left.y
            + self.rect.size.height as i32 / 2
            + font.character_size.height as i32 / 3;

        renderer.draw_text(
            &self.label,
            Point::new(center_x, center_y),
            font,
            appearance.text,
            Alignment::Center,
        )?;

        Ok(())
    }
}
