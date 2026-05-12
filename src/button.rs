//! Tappable rectangle with a label that emits a Message when pressed.

use crate::{Renderer, Theme, Widget, renderer::RenderError, theme::ButtonStyle};
use alloc::string::String;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};

/// Tappable button with a label.
///
/// `M: Clone` is your application's message type. Configure with
/// `.on_press(msg)`. Option per-instance style override via `.style(s)`.
pub struct Button<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    label: String,
    on_press: Option<M>,
    pressed: bool,
    style_override: Option<ButtonStyle<'a, C>>,
}

impl<'a, C: PixelColor, M: Clone> Button<'a, C, M> {
    /// Create a new button.
    pub fn new(rect: Rectangle, label: impl Into<String>) -> Self {
        Self {
            rect,
            label: label.into(),
            on_press: None,
            pressed: false,
            style_override: None,
        }
    }

    /// Builder: set the message emitted on tap.
    pub fn on_press(mut self, msg: M) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Builder: replace the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Builder: override the theme's button style for this instance.
    pub fn style(mut self, style: ButtonStyle<'a, C>) -> Self {
        self.style_override = Some(style);
        self
    }

    /// Manually clear the pressed state (call after handling the message).
    pub fn release(&mut self) {
        self.pressed = false;
    }

    fn resolved_style<'t>(&'t self, theme: &'t Theme<'a, C>) -> &'t ButtonStyle<'a, C> {
        self.style_override.as_ref().unwrap_or(&theme.button)
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

impl<'a, C: PixelColor, M: Clone> Widget<'a, C, M> for Button<'a, C, M> {
    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn set_rect(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn handle_touch(&mut self, point: Point) -> Option<M> {
        if self.hit_test(point) {
            self.pressed = true;
            self.on_press.clone()
        } else {
            None
        }
    }

    fn draw(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'a, C>,
    ) -> Result<(), RenderError> {
        let style = self.resolved_style(theme);
        let bg = if self.pressed {
            style.pressed_background
        } else {
            style.background
        };

        renderer.fill_rect(self.rect, bg)?;
        renderer.stroke_rect(self.rect, style.border)?;

        let center_x = self.rect.top_left.x + self.rect.size.width as i32 / 2;
        let center_y = self.rect.top_left.y
            + self.rect.size.height as i32 / 2
            + style.font.character_size.height as i32 / 3;

        renderer.draw_text(
            &self.label,
            Point::new(center_x, center_y),
            style.font,
            style.label,
            Alignment::Center,
        )?;

        Ok(())
    }
}
