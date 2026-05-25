//! Tappable button that draws a borrowed raster image instead of (or
//! alongside) a text label.
//!
//! Interaction mirrors [`Button`](super::button::Button): the press
//! registers on [`TouchPhase::Down`] (sets `pressed`, emits no message),
//! and the message fires on [`TouchPhase::Up`] if `pressed` is still set.
//! The runtime re-marks `pressed` each frame via
//! [`mark_pressed`](Widget::mark_pressed), so the press survives the Down/Up
//! gesture; dragging off before release cancels it, since a press that no
//! longer lands isn't re-asserted on the next rebuild.
//!
//! Styling is resolved through the theme's [`ButtonCatalog`] using the
//! widget's [`ButtonClass`] and current [`Status`], exactly like
//! [`Button`](super::button::Button). The image is blitted via
//! [`Renderer::draw_image`] centered in the upper part of the button,
//! with an optional label drawn beneath it.

use super::Widget;
use alloc::string::String;
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::{ButtonCatalog, ButtonClass, Status, Theme};

/// Tappable button whose face is a borrowed raster image, with an
/// optional text label below it.
pub struct ImageButton<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    pixels: &'a [C],
    image_size: Size,
    label: Option<String>,
    on_press: Option<M>,
    pressed: bool,
    class: ButtonClass,
    font_override: Option<&'a MonoFont<'a>>,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> ImageButton<'a, C, M> {
    /// Create a new image button from a row-major `pixels` slice of
    /// `image_size`. Defaults to [`ButtonClass::Standard`] and no label.
    /// Position and size are assigned by the parent container via
    /// `arrange`.
    pub fn new(pixels: &'a [C], image_size: Size) -> Self {
        Self {
            rect: Rectangle::zero(),
            pixels,
            image_size,
            label: None,
            on_press: None,
            pressed: false,
            class: ButtonClass::Standard,
            font_override: None,
            width: Length::Shrink,
            height: Length::Shrink,
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

    /// Add a text label drawn beneath the image.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Select the semantic [`ButtonClass`]. Default is
    /// `Standard`.
    #[must_use]
    pub fn class(mut self, class: ButtonClass) -> Self {
        self.class = class;
        self
    }

    /// Override the default font for the label.
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font_override = Some(font);
        self
    }

    /// True iff this button has a message bound. Disabled buttons render
    /// dimmed and ignore touches.
    pub fn is_enabled(&self) -> bool {
        self.on_press.is_some()
    }

    fn status(&self) -> Status {
        if !self.is_enabled() {
            Status::Disabled
        } else if self.pressed {
            Status::Pressed
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

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for ImageButton<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        // The label font is unknown until draw; estimate height from the
        // image plus a generous text line so the slot never clips.
        let label_h: u32 = if self.label.is_some() { 20 } else { 0 };
        let intrinsic = Size::new(
            self.image_size.width + 16,
            self.image_size.height + label_h + 16,
        );
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

        // Vertical content layout: image on top, optional label beneath.
        let label_h = if self.label.is_some() {
            font.character_size.height as i32 + 4
        } else {
            0
        };
        let content_h = self.image_size.height as i32 + label_h;
        let avail_h = self.rect.size.height as i32;
        let top = self.rect.top_left.y + (avail_h - content_h).max(0) / 2;

        // Center the image horizontally within the button.
        let dx = (self.rect.size.width as i32 - self.image_size.width as i32).max(0) / 2;
        let origin = Point::new(self.rect.top_left.x + dx, top);
        renderer.draw_image(origin, self.image_size, self.pixels)?;

        if let Some(label) = &self.label {
            let center_x = self.rect.top_left.x + self.rect.size.width as i32 / 2;
            let baseline_y =
                top + self.image_size.height as i32 + font.character_size.height as i32;
            renderer.draw_text(
                label,
                Point::new(center_x, baseline_y),
                font,
                appearance.text,
                Alignment::Center,
            )?;
        }

        Ok(())
    }
}
