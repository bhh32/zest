//! Toggleable check box with an optional trailing label.
//!
//! Immediate-mode: the host owns the `bool` and rebuilds the widget each
//! frame, passing the current value to [`Checkbox::new`]. A tap emits
//! `on_toggle(!checked)` so the host can flip its stored value.
//!
//! Click semantics mirror [`Button`](crate::Button): the press registers
//! on `TouchPhase::Down` (visual feedback only), and the toggle message
//! fires on `TouchPhase::Up` if the press is still active. The runtime
//! rehydrates the pressed flag each frame via
//! [`mark_pressed`](Widget::mark_pressed), so drag-off-to-cancel works.
//!
//! Colors come from the theme's accent [`Component`](zest_theme::Component):
//! the filled box uses `accent.base`/`accent.pressed`, the check glyph uses
//! `accent.on_base`, and the unchecked box border uses `accent.border`.

use super::Widget;
use alloc::{boxed::Box, string::String};
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Side length of the box glyph in pixels.
const BOX_SIZE: u32 = 20;
/// Gap between the box and the label in pixels.
const LABEL_GAP: u32 = 6;

/// Toggleable check box. Host owns the `bool`; a tap emits the negated
/// value through [`on_toggle`](Checkbox::on_toggle).
pub struct Checkbox<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    checked: bool,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> M + 'a>>,
    pressed: bool,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> Checkbox<'a, C, M> {
    /// New check box reflecting `checked`. Position and size are assigned
    /// by the parent container via `arrange`.
    pub fn new(checked: bool) -> Self {
        Self {
            rect: Rectangle::zero(),
            checked,
            label: None,
            on_toggle: None,
            pressed: false,
            width: Length::Shrink,
            height: Length::Fixed(BOX_SIZE),
            _color: PhantomData,
        }
    }

    /// Trailing label drawn to the right of the box.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Callback invoked with the toggled value (`!checked`) on
    /// each tap. Without it the check box is disabled and ignores touches.
    #[must_use]
    pub fn on_toggle<F: Fn(bool) -> M + 'a>(mut self, f: F) -> Self {
        self.on_toggle = Some(Box::new(f));
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

    /// True iff a toggle callback is bound. Disabled check boxes render
    /// dimmed and ignore touches.
    pub fn is_enabled(&self) -> bool {
        self.on_toggle.is_some()
    }

    fn intrinsic(&self) -> Size {
        // Approximate label width with a fixed glyph advance — no theme
        // (font) reference is available at measure time.
        let label_w = self
            .label
            .as_ref()
            .map_or(0, |l| LABEL_GAP + l.chars().count() as u32 * 8);
        Size::new(BOX_SIZE + label_w, BOX_SIZE)
    }

    fn box_rect(&self) -> Rectangle {
        let y = self.rect.top_left.y + (self.rect.size.height.saturating_sub(BOX_SIZE) / 2) as i32;
        Rectangle::new(
            Point::new(self.rect.top_left.x, y),
            Size::new(BOX_SIZE, BOX_SIZE),
        )
    }

    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Checkbox<'a, C, M> {
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
                    self.on_toggle.as_ref().map(|cb| cb(!self.checked))
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
        let accent = &theme.accent;
        let box_rect = self.box_rect();

        if self.checked {
            let fill = if self.pressed {
                accent.pressed
            } else {
                accent.base
            };
            renderer.fill_rect(box_rect, fill)?;
            renderer.stroke_rect(box_rect, accent.border)?;
            // Draw a check mark as two strokes forming a tick.
            let x = box_rect.top_left.x;
            let y = box_rect.top_left.y;
            let s = BOX_SIZE as i32;
            renderer.stroke_line(
                Point::new(x + s * 3 / 16, y + s / 2),
                Point::new(x + s * 7 / 16, y + s * 11 / 16),
                accent.on_base,
                2,
            )?;
            renderer.stroke_line(
                Point::new(x + s * 7 / 16, y + s * 11 / 16),
                Point::new(x + s * 13 / 16, y + s * 5 / 16),
                accent.on_base,
                2,
            )?;
        } else {
            let bg = if self.pressed {
                accent.pressed
            } else {
                theme.background.base
            };
            renderer.fill_rect(box_rect, bg)?;
            renderer.stroke_rect(box_rect, accent.border)?;
        }

        if let Some(label) = &self.label {
            let font = theme.default_font();
            let text_x = box_rect.top_left.x + BOX_SIZE as i32 + LABEL_GAP as i32;
            let center_y = self.rect.top_left.y
                + self.rect.size.height as i32 / 2
                + font.character_size.height as i32 / 3;
            let color = if self.is_enabled() {
                theme.background.on_base
            } else {
                theme.palette.neutral_2
            };
            renderer.draw_text(
                label,
                Point::new(text_x, center_y),
                font,
                color,
                Alignment::Left,
            )?;
        }

        Ok(())
    }
}
