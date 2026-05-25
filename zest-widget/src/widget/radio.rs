//! Radio button: an outer circle with an inner dot when selected, plus an
//! optional trailing label.
//!
//! Immediate-mode: the host owns the selected state and rebuilds the widget
//! each frame, passing whether *this* button is selected to
//! [`RadioButton::new`]. A tap emits the [`on_select`](RadioButton::on_select)
//! message. **Exclusivity is the host's responsibility**: render one
//! `RadioButton` per option, set `selected = (option == current)`, and in the
//! handler for each button's message store that option as the new current.
//! The widget itself knows nothing about its siblings.
//!
//! Click semantics mirror [`Button`](crate::Button): the press registers on
//! `TouchPhase::Down` and the message fires on `TouchPhase::Up` if the press
//! is still active. The runtime rehydrates the pressed flag each frame via
//! [`mark_pressed`](Widget::mark_pressed).
//!
//! Colors come from the theme's accent [`Component`](zest_theme::Component):
//! the inner dot uses `accent.base`, the outer ring uses `accent.border`,
//! and the label uses `background.on_base`.

use super::Widget;
use alloc::string::String;
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Outer circle diameter in pixels.
const CIRCLE_SIZE: u32 = 20;
/// Gap between the circle and the label in pixels.
const LABEL_GAP: u32 = 6;

/// Radio button. Host owns selection and exclusivity; a tap emits
/// [`on_select`](RadioButton::on_select).
pub struct RadioButton<C: PixelColor, M: Clone> {
    rect: Rectangle,
    selected: bool,
    label: Option<String>,
    on_select: Option<M>,
    pressed: bool,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<C: PixelColor, M: Clone> RadioButton<C, M> {
    /// New radio button reflecting `selected`. Position and size are
    /// assigned by the parent container via `arrange`.
    pub fn new(selected: bool) -> Self {
        Self {
            rect: Rectangle::zero(),
            selected,
            label: None,
            on_select: None,
            pressed: false,
            width: Length::Shrink,
            height: Length::Fixed(CIRCLE_SIZE),
            _color: PhantomData,
        }
    }

    /// Trailing label drawn to the right of the circle.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Message emitted on tap. Without it the button is disabled
    /// and ignores touches.
    #[must_use]
    pub fn on_select(mut self, msg: M) -> Self {
        self.on_select = Some(msg);
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

    /// True iff a select message is bound. Disabled buttons render dimmed
    /// and ignore touches.
    pub fn is_enabled(&self) -> bool {
        self.on_select.is_some()
    }

    fn intrinsic(&self) -> Size {
        let label_w = self
            .label
            .as_ref()
            .map_or(0, |l| LABEL_GAP + l.chars().count() as u32 * 8);
        Size::new(CIRCLE_SIZE + label_w, CIRCLE_SIZE)
    }

    /// Center of the outer circle, vertically centered within `rect`.
    fn circle_center(&self) -> Point {
        let r = CIRCLE_SIZE as i32 / 2;
        Point::new(
            self.rect.top_left.x + r,
            self.rect.top_left.y + self.rect.size.height as i32 / 2,
        )
    }

    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for RadioButton<C, M> {
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
                    self.on_select.clone()
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
        let center = self.circle_center();
        let outer = CIRCLE_SIZE / 2;

        // Outer ring: fill border color, then punch out the interior so a
        // ring remains (the renderer has no stroke_circle primitive).
        renderer.fill_circle(center, outer, accent.border)?;
        let interior_color = if self.pressed {
            accent.pressed
        } else {
            theme.background.base
        };
        renderer.fill_circle(center, outer.saturating_sub(2), interior_color)?;

        // Inner dot when selected.
        if self.selected {
            let dot = if self.is_enabled() {
                accent.base
            } else {
                theme.background.divider
            };
            renderer.fill_circle(center, outer.saturating_sub(6), dot)?;
        }

        if let Some(label) = &self.label {
            let font = theme.default_font();
            let text_x = self.rect.top_left.x + CIRCLE_SIZE as i32 + LABEL_GAP as i32;
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
