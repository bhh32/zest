//! Drop-down selector: a button-like field showing the current selection
//! that, when open, reveals its option list as an overlay.
//!
//! Immediate-mode: the host owns *both* the open flag and the selected
//! index and rebuilds the widget each frame, passing the current values to
//! [`Dropdown::open`] and [`Dropdown::selected`]. The widget emits messages
//! and never mutates either piece of state itself:
//!
//! * Tapping the field emits [`on_toggle`](Dropdown::on_toggle) with the
//!   negated open flag, so the host flips its stored `bool`.
//! * Tapping an option emits [`on_select`](Dropdown::on_select) with that
//!   option's index, so the host stores it (and typically closes the list).
//!
//! ## How the overlay works
//!
//! The whole widget is assembled as a [`Stack`](crate::Stack): the field is
//! the bottom layer, and — only while `open` — the option list is pushed
//! *last* so it draws on top of and intercepts touch before everything
//! beneath it (see [`stack`](super::stack) for the z-order rules). The list
//! is aligned to the top of the stack region, directly under where the
//! field sits, so it reads as "dropping down" from the field. Because the
//! host owns the open flag, toggling it simply adds or removes that final
//! layer on the next frame.
//!
//! The stack is composed lazily on the first lifecycle call, so the
//! chainable builders only record configuration — keeping each
//! `Box<dyn Fn>` callback single-owned (callbacks cannot be cloned).
//!
//! Colors come from the theme: the field reuses the standard
//! [`button`](zest_theme::Theme::button) component styling, the open list
//! paints a [`background`](zest_theme::Theme::background)-based card, and the
//! currently-selected row is highlighted with the
//! [`accent`](zest_theme::Theme::accent) color.

use super::{Widget, element::Element};
use alloc::{boxed::Box, string::String, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Horizontal, Length, RenderError, Renderer, TouchPhase, Vertical};
use zest_theme::Theme;

/// Default height of the field and of each option row, in pixels.
const ROW_HEIGHT: u32 = 36;
/// Horizontal text inset inside the field and option rows, in pixels.
const TEXT_PAD: i32 = 8;

/// A drop-down selector. The host owns the open flag and selected index;
/// the widget renders the field, draws the option list as an overlay while
/// open, and emits [`on_toggle`](Dropdown::on_toggle) /
/// [`on_select`](Dropdown::on_select).
pub struct Dropdown<'a, C: PixelColor, M: Clone> {
    options: Vec<String>,
    selected: usize,
    is_open: bool,
    placeholder: String,
    on_toggle: Option<Box<dyn Fn(bool) -> M + 'a>>,
    on_select: Option<Box<dyn Fn(usize) -> M + 'a>>,
    width: Length,
    height: Length,
    /// Composed stack, built on first lifecycle call. `None` until then.
    stack: Option<Element<'a, C, M>>,
    /// Cached option count (the list owns its own copy of `options` once
    /// built, so `arrange` reads the row count from here).
    option_count: usize,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Dropdown<'a, C, M> {
    /// New drop-down. Position and size are assigned by the parent
    /// container via `arrange`. Defaults: no options, selection 0, closed,
    /// fill width, 36px field height.
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: 0,
            is_open: false,
            placeholder: String::new(),
            on_toggle: None,
            on_select: None,
            width: Length::Fill,
            height: Length::Fixed(ROW_HEIGHT),
            stack: None,
            option_count: 0,
        }
    }

    /// Builder: width sizing intent (default [`Length::Fill`]).
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: height sizing intent of the *field* (default 36px). The
    /// option list always uses one fixed-height row per option.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Builder: the option labels. They are copied into owned `String`s so
    /// the widget can outlive the borrowed slice.
    #[must_use]
    pub fn options(mut self, options: &[&str]) -> Self {
        self.options = options.iter().map(|s| String::from(*s)).collect();
        self.option_count = self.options.len();
        self
    }

    /// Builder: the currently-selected option index (host-owned).
    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Builder: whether the option list is currently shown (host-owned).
    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Builder: text shown on the field when the selected index is out of
    /// range (e.g. nothing selected yet).
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Builder: callback invoked when the field is tapped, receiving the
    /// negated open flag. Without it the field does not toggle.
    #[must_use]
    pub fn on_toggle<F: Fn(bool) -> M + 'a>(mut self, f: F) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Builder: callback invoked when an option is tapped, receiving that
    /// option's index. Without it the options are inert.
    #[must_use]
    pub fn on_select<F: Fn(usize) -> M + 'a>(mut self, f: F) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Compose the field (and, while open, the option list) into a
    /// [`Stack`](crate::Stack), consuming the recorded config. Callbacks
    /// are moved out via [`Option::take`] so each is owned by exactly one
    /// inner widget.
    fn build(&mut self) -> Element<'a, C, M> {
        let label = self
            .options
            .get(self.selected)
            .cloned()
            .unwrap_or_else(|| self.placeholder.clone());

        let field = DropdownField {
            rect: Rectangle::zero(),
            label,
            open: self.is_open,
            on_toggle: self.on_toggle.take(),
            pressed: false,
            width: self.width,
            height: self.height,
            _color: PhantomData,
        };

        let mut stack = super::stack::Stack::new()
            .width(self.width)
            .height(self.height);
        stack = stack.push_aligned(field, Horizontal::Left, Vertical::Top);

        if self.is_open && !self.options.is_empty() {
            let list = DropdownList {
                rect: Rectangle::zero(),
                options: self.options.clone(),
                selected: self.selected,
                on_select: self.on_select.take(),
                pressed: None,
                _color: PhantomData,
            };
            // Push last → drawn on top, touched first; aligned under the
            // field at the top of the stack region.
            stack = stack.push_aligned(list, Horizontal::Left, Vertical::Top);
        }

        Element::new(stack)
    }

    /// Build the stack on demand if it hasn't been composed yet.
    fn ensure_built(&mut self) {
        if self.stack.is_none() {
            self.stack = Some(self.build());
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Dropdown<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Dropdown<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        self.ensure_built();
        // Report only the field's slot to the parent; the open list is an
        // overlay that paints outside the reported size (like any modal).
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
        let h = self.height.resolve(ROW_HEIGHT, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.ensure_built();
        // Arrange the stack over a region tall enough for the field plus
        // the open list, so the overlay rows get real rects. The field is
        // top-aligned and keeps its own height inside this region.
        let extra = if self.is_open {
            ROW_HEIGHT.saturating_mul(self.option_count as u32)
        } else {
            0
        };
        let region = Rectangle::new(
            rect.top_left,
            Size::new(rect.size.width, rect.size.height + extra),
        );
        if let Some(stack) = self.stack.as_mut() {
            stack.arrange(region);
        }
    }

    fn rect(&self) -> Rectangle {
        self.stack.as_ref().map_or(Rectangle::zero(), |s| s.rect())
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.stack
            .as_mut()
            .and_then(|s| s.handle_touch(point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(stack) = self.stack.as_mut() {
            stack.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        if let Some(stack) = &self.stack {
            stack.draw(renderer, theme)?;
        }
        Ok(())
    }
}

// ---- internal: the bottom-layer field --------------------------------------

/// The always-present field: a button-like rect showing the current
/// selection plus a caret. Emits the toggle callback on tap.
struct DropdownField<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    label: String,
    open: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> M + 'a>>,
    pressed: bool,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<C: PixelColor, M: Clone> DropdownField<'_, C, M> {
    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for DropdownField<'_, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
        let h = self.height.resolve(ROW_HEIGHT, constraints.max.height);
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
        if self.on_toggle.is_none() || !self.hit_test(point) {
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
                    let open = self.open;
                    self.on_toggle.as_ref().map(|cb| cb(!open))
                } else {
                    None
                }
            }
            TouchPhase::Moved => None,
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.on_toggle.is_some() && self.hit_test(point) {
            self.pressed = true;
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let comp = &theme.button;
        let bg = if self.pressed { comp.pressed } else { comp.base };
        renderer.fill_rect(self.rect, bg)?;
        renderer.stroke_rect(self.rect, comp.border)?;

        let font = theme.default_font();
        let text_y = self.rect.top_left.y
            + self.rect.size.height as i32 / 2
            + font.character_size.height as i32 / 3;
        renderer.draw_text(
            &self.label,
            Point::new(self.rect.top_left.x + TEXT_PAD, text_y),
            font,
            comp.on_base,
            Alignment::Left,
        )?;

        // Caret on the right: a small triangle, flipped when open. Drawn as
        // three strokes (the renderer has no triangle primitive).
        let cx = self.rect.top_left.x + self.rect.size.width as i32 - TEXT_PAD - 6;
        let cy = self.rect.top_left.y + self.rect.size.height as i32 / 2;
        if self.open {
            renderer.stroke_line(Point::new(cx, cy + 3), Point::new(cx + 6, cy + 3), comp.on_base, 2)?;
            renderer.stroke_line(Point::new(cx, cy + 3), Point::new(cx + 3, cy - 3), comp.on_base, 2)?;
            renderer.stroke_line(Point::new(cx + 6, cy + 3), Point::new(cx + 3, cy - 3), comp.on_base, 2)?;
        } else {
            renderer.stroke_line(Point::new(cx, cy - 3), Point::new(cx + 6, cy - 3), comp.on_base, 2)?;
            renderer.stroke_line(Point::new(cx, cy - 3), Point::new(cx + 3, cy + 3), comp.on_base, 2)?;
            renderer.stroke_line(Point::new(cx + 6, cy - 3), Point::new(cx + 3, cy + 3), comp.on_base, 2)?;
        }
        Ok(())
    }
}

// ---- internal: the open option list overlay --------------------------------

/// The drop-down list shown while open: one row per option, the selected
/// row highlighted, each row emitting the select callback on tap.
struct DropdownList<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    options: Vec<String>,
    selected: usize,
    on_select: Option<Box<dyn Fn(usize) -> M + 'a>>,
    /// Index of the row currently held down (for pressed feedback).
    pressed: Option<usize>,
    _color: PhantomData<C>,
}

impl<C: PixelColor, M: Clone> DropdownList<'_, C, M> {
    fn list_height(&self) -> u32 {
        ROW_HEIGHT.saturating_mul(self.options.len() as u32)
    }

    /// The row index containing `point`, if any.
    fn row_at(&self, point: Point) -> Option<usize> {
        let tl = self.rect.top_left;
        if point.x < tl.x || point.x >= tl.x + self.rect.size.width as i32 {
            return None;
        }
        let dy = point.y - tl.y;
        if dy < 0 {
            return None;
        }
        let idx = (dy as u32 / ROW_HEIGHT) as usize;
        (idx < self.options.len()).then_some(idx)
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for DropdownList<'_, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = constraints.max.width;
        let h = self.list_height().min(constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (Length::Fill, Length::Fixed(self.list_height()))
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        let row = self.row_at(point);
        match phase {
            TouchPhase::Down => {
                self.pressed = row;
                None
            }
            TouchPhase::Up => {
                let armed = self.pressed.take();
                match (row, armed) {
                    (Some(r), Some(a)) if r == a => self.on_select.as_ref().map(|cb| cb(r)),
                    _ => None,
                }
            }
            TouchPhase::Moved => {
                if row != self.pressed {
                    self.pressed = None;
                }
                None
            }
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(r) = self.row_at(point) {
            self.pressed = Some(r);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let bg = theme.background.base;
        let font = theme.default_font();
        renderer.fill_rect(self.rect, bg)?;
        renderer.stroke_rect(self.rect, theme.button.border)?;

        let x = self.rect.top_left.x;
        let w = self.rect.size.width;
        for (i, opt) in self.options.iter().enumerate() {
            let y = self.rect.top_left.y + (i as u32 * ROW_HEIGHT) as i32;
            let row_rect = Rectangle::new(Point::new(x, y), Size::new(w, ROW_HEIGHT));

            let highlighted = self.pressed == Some(i);
            let selected = i == self.selected;
            if highlighted {
                renderer.fill_rect(row_rect, theme.accent.pressed)?;
            } else if selected {
                renderer.fill_rect(row_rect, theme.accent.base)?;
            }

            let text_color = if highlighted || selected {
                theme.accent.on_base
            } else {
                theme.background.on_base
            };
            let text_y = y + ROW_HEIGHT as i32 / 2 + font.character_size.height as i32 / 3;
            renderer.draw_text(
                opt,
                Point::new(x + TEXT_PAD, text_y),
                font,
                text_color,
                Alignment::Left,
            )?;

            // Separator between rows (skip after the last).
            if i + 1 < self.options.len() {
                let sep_y = y + ROW_HEIGHT as i32 - 1;
                renderer.fill_rect(
                    Rectangle::new(Point::new(x, sep_y), Size::new(w, 1)),
                    theme.background.divider,
                )?;
            }
        }
        Ok(())
    }
}
