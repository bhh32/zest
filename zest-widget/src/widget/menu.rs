//! Vertical menu: a stack of selectable entries, each emitting a message
//! when tapped.
//!
//! Immediate-mode: the host owns which entry is currently selected and
//! passes it via [`Menu::selected`]; the selected entry is highlighted
//! with the accent color. Tapping an entry emits the message bound to it,
//! using the same press-then-release semantics as [`Button`](super::Button)
//! (a press is only fired if finger-down and finger-up land on the same
//! entry).

use super::Widget;
use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Default per-entry height in pixels.
const ROW_H: u32 = 30;
/// Horizontal label inset in pixels.
const PAD_X: i32 = 10;

/// A single menu entry: a label and the message emitted when tapped.
struct Entry<M> {
    label: String,
    message: M,
}

/// Vertical menu of selectable entries.
pub struct Menu<C: PixelColor, M: Clone> {
    rect: Rectangle,
    entries: Vec<Entry<M>>,
    selected: Option<usize>,
    pressed: Option<usize>,
    row_h: u32,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<C: PixelColor, M: Clone> Menu<C, M> {
    /// Create an empty menu. Add entries with [`entry`](Self::entry).
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            entries: Vec::new(),
            selected: None,
            pressed: None,
            row_h: ROW_H,
            width: Length::Fill,
            height: Length::Shrink,
            _color: PhantomData,
        }
    }

    /// Builder: append an entry with the message emitted when it is tapped.
    #[must_use]
    pub fn entry(mut self, label: impl Into<String>, message: M) -> Self {
        self.entries.push(Entry {
            label: label.into(),
            message,
        });
        self
    }

    /// Builder: index of the currently-selected entry (highlighted).
    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// Builder: per-row height in pixels (default 30).
    #[must_use]
    pub fn row_height(mut self, h: u32) -> Self {
        self.row_h = h.max(1);
        self
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    fn row_rect(&self, i: usize) -> Rectangle {
        Rectangle::new(
            self.rect.top_left + Point::new(0, (i as u32 * self.row_h) as i32),
            Size::new(self.rect.size.width, self.row_h),
        )
    }

    fn row_at(&self, p: Point) -> Option<usize> {
        let left = self.rect.top_left.x;
        if p.x < left || p.x >= left + self.rect.size.width as i32 {
            return None;
        }
        let dy = p.y - self.rect.top_left.y;
        if dy < 0 {
            return None;
        }
        let idx = (dy as u32 / self.row_h) as usize;
        (idx < self.entries.len()).then_some(idx)
    }
}

impl<C: PixelColor, M: Clone> Default for Menu<C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for Menu<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let intrinsic_h = self.row_h * self.entries.len() as u32;
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
        let h = self.height.resolve(intrinsic_h, constraints.max.height);
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
        match phase {
            TouchPhase::Down => {
                self.pressed = self.row_at(point);
                None
            }
            TouchPhase::Moved => {
                if self.row_at(point) != self.pressed {
                    self.pressed = None;
                }
                None
            }
            TouchPhase::Up => {
                let now = self.row_at(point);
                let pressed = self.pressed.take();
                if let (Some(i), Some(p)) = (now, pressed) {
                    if i == p {
                        return Some(self.entries[i].message.clone());
                    }
                }
                None
            }
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.pressed.is_none() {
            self.pressed = self.row_at(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let font = theme.default_font();
        let glyph_h = font.character_size.height as i32;
        for (i, e) in self.entries.iter().enumerate() {
            let r = self.row_rect(i);
            let (bg, fg) = if Some(i) == self.selected {
                (theme.accent.base, theme.accent.on_base)
            } else if Some(i) == self.pressed {
                (theme.button.pressed, theme.button.on_base)
            } else {
                (theme.button.base, theme.button.on_base)
            };
            renderer.fill_rect(r, bg)?;
            renderer.stroke_rect(r, theme.button.border)?;
            renderer.draw_text(
                &e.label,
                Point::new(
                    r.top_left.x + PAD_X,
                    r.top_left.y + self.row_h as i32 / 2 + glyph_h / 3,
                ),
                font,
                fg,
                Alignment::Left,
            )?;
        }
        Ok(())
    }
}
