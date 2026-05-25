//! On-screen keyboard modeled on LVGL's `lv_keyboard`.
//!
//! The keyboard has four keymaps ([`KeyboardMode`]) — lowercase text,
//! uppercase text, special symbols, and a numeric pad — mirroring LVGL's
//! `LV_KEYBOARD_MODE_*`. The mode keys (`1#`, `ABC`/`abc`) switch the active
//! keymap *in place*: uppercase is a sticky mode like LVGL's, not a one-shot
//! shift. Character keys emit [`KeyAction::Char`]; the control keys emit
//! backspace, newline, cursor-left / cursor-right, OK ([`KeyAction::Ready`])
//! and hide ([`KeyAction::Cancel`]) — the same control set LVGL's keyboard
//! sends to its attached text area.
//!
//! In the transient widget model the *host* owns the current
//! [`KeyboardMode`] (and the edited text). Each frame `view()` builds a fresh
//! `Keyboard` from the current mode, and the host's `update` applies each
//! [`KeyAction`] — inserting / deleting characters, moving the cursor, and
//! switching mode on [`KeyAction::Mode`]. Pair it with a
//! [`TextArea`](super::text_area::TextArea) for an LVGL keyboard+textarea
//! setup; the built-in preview field is off by default (enable it with
//! [`show_field`](Keyboard::show_field) for a standalone keyboard).

use super::{Widget, button::Button, column::Column, row::Row};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::{ButtonClass, Theme};

/// Reserved height (px) for the optional title + preview field.
const FIELD_H: u32 = 70;
/// Width (px) of the password Show/Hide toggle button in the preview field.
const SHOW_W: u32 = 56;

/// Keyboard keymap, mirroring LVGL's `lv_keyboard_mode_t`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardMode {
    /// Lowercase QWERTY letters.
    TextLower,
    /// Uppercase QWERTY letters.
    TextUpper,
    /// Special symbols and punctuation.
    Special,
    /// Numeric pad.
    Number,
}

/// An action emitted by a key press. The host applies it in `update`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    /// Insert this character at the cursor.
    Char(char),
    /// Delete the character before the cursor.
    Backspace,
    /// Insert a line break (or submit, for single-line fields).
    Newline,
    /// Move the cursor one character left.
    CursorLeft,
    /// Move the cursor one character right.
    CursorRight,
    /// Switch to a different keymap. The host stores the new
    /// [`KeyboardMode`] and passes it back next frame.
    Mode(KeyboardMode),
    /// Accept / confirm (LVGL `LV_EVENT_READY`).
    Ready,
    /// Hide / cancel (LVGL `LV_EVENT_CANCEL`).
    Cancel,
    /// Tapped the password field's Show/Hide button. The host flips its
    /// `reveal` flag and passes it back via [`Keyboard::reveal`].
    ToggleReveal,
}

/// A single key in a keymap row: a label, the action it emits, and a flex
/// weight (control keys are wider than character keys).
struct Key {
    label: String,
    action: KeyAction,
    weight: u32,
}

/// A character key (flex weight 1).
fn ch(c: char) -> Key {
    Key {
        label: c.to_string(),
        action: KeyAction::Char(c),
        weight: 1,
    }
}

/// A control key with an explicit label and flex weight.
fn ctrl(label: &str, action: KeyAction, weight: u32) -> Key {
    Key {
        label: label.to_string(),
        action,
        weight,
    }
}

/// The shared bottom row for the text/special keymaps:
/// hide | cursor-left | space | cursor-right | OK.
fn bottom_row() -> Vec<Key> {
    vec![
        ctrl("▼", KeyAction::Cancel, 2),
        ctrl("←", KeyAction::CursorLeft, 1),
        ctrl("space", KeyAction::Char(' '), 6),
        ctrl("→", KeyAction::CursorRight, 1),
        ctrl("OK", KeyAction::Ready, 2),
    ]
}

/// On-screen keyboard. Built fresh each frame from the host-owned
/// [`KeyboardMode`].
pub struct Keyboard<'a, C: PixelColor, M: Clone> {
    bounds: Rectangle,
    title: String,
    input: String,
    is_password: bool,
    show_field: bool,
    reveal: bool,
    keys: Column<'a, C, M>,
    width: Length,
    height: Length,
    /// Host message emitted when the password Show/Hide button is tapped.
    toggle_reveal: M,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Keyboard<'a, C, M> {
    /// Construct a keyboard for `mode`. `on_action` translates each
    /// [`KeyAction`] into the host's message type. Position and size are
    /// assigned by the parent via `arrange`.
    pub fn new<F>(mode: KeyboardMode, on_action: F) -> Self
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let toggle_reveal = on_action(KeyAction::ToggleReveal);
        let keys = Self::build_keys(mode, on_action);
        Self {
            bounds: Rectangle::zero(),
            title: String::new(),
            input: String::new(),
            is_password: false,
            show_field: false,
            reveal: false,
            keys,
            width: Length::Fill,
            height: Length::Fill,
            toggle_reveal,
        }
    }

    /// Builder: title shown above the optional preview field.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: text shown in the optional preview field.
    #[must_use]
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.input = input.into();
        self
    }

    /// Builder: render the preview field's text masked as `*`, with a
    /// Show/Hide button so the user can verify what they typed on the small
    /// keys. Only takes effect with [`show_field`](Self::show_field) on.
    #[must_use]
    pub fn is_password(mut self, is_password: bool) -> Self {
        self.is_password = is_password;
        self
    }

    /// Builder: reveal the password text (host-owned). Tapping the Show/Hide
    /// button emits [`KeyAction::ToggleReveal`]; the host flips its flag and
    /// passes it back here so the field unmasks.
    #[must_use]
    pub fn reveal(mut self, reveal: bool) -> Self {
        self.reveal = reveal;
        self
    }

    /// Builder: show the built-in title + preview field at the top (default
    /// `false`). Leave off when pairing with a
    /// [`TextArea`](super::text_area::TextArea); turn on for a standalone
    /// keyboard that displays its own input.
    #[must_use]
    pub fn show_field(mut self, show: bool) -> Self {
        self.show_field = show;
        self
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn key_bounds_for(&self, bounds: Rectangle) -> Rectangle {
        let reserve: u32 = if self.show_field { FIELD_H } else { 0 };
        Rectangle::new(
            Point::new(bounds.top_left.x, bounds.top_left.y + reserve as i32),
            Size::new(bounds.size.width, bounds.size.height.saturating_sub(reserve)),
        )
    }

    /// Rect of the password Show/Hide button inside the preview field, or
    /// `None` when there is no field or it isn't a password keyboard.
    fn show_button_rect(&self) -> Option<Rectangle> {
        if !(self.is_password && self.show_field) {
            return None;
        }
        let x0 = self.bounds.top_left.x;
        let y0 = self.bounds.top_left.y;
        let width = self.bounds.size.width;
        Some(Rectangle::new(
            Point::new(x0 + width as i32 - 8 - SHOW_W as i32, y0 + 30),
            Size::new(SHOW_W, 28),
        ))
    }

    fn build_keys<F>(mode: KeyboardMode, on_action: F) -> Column<'a, C, M>
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let mut col = Column::new().spacing(2);
        for row in keymap(mode) {
            col = col.push(Self::build_row(row, on_action));
        }
        col
    }

    fn build_row<F>(spec: Vec<Key>, on_action: F) -> Row<'a, C, M>
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let mut row = Row::new().spacing(2);
        for key in spec {
            let class = match key.action {
                KeyAction::Ready => ButtonClass::Suggested,
                KeyAction::Cancel => ButtonClass::Destructive,
                _ => ButtonClass::Standard,
            };
            row = row.push(
                Button::new(key.label)
                    .on_press(on_action(key.action))
                    .class(class)
                    .width(Length::FillPortion(key.weight)),
            );
        }
        row
    }
}

/// The keymap (rows of keys) for a given mode, following LVGL's layouts.
fn keymap(mode: KeyboardMode) -> Vec<Vec<Key>> {
    use KeyAction::{Backspace, CursorLeft, CursorRight, Mode, Newline, Ready};
    use KeyboardMode::{Number, Special, TextLower, TextUpper};

    /// Helper: build a row from a leading control key, a run of chars, and a
    /// trailing control key.
    fn text_row(lead: Key, mids: &str, tail: Key) -> Vec<Key> {
        let mut r = vec![lead];
        r.extend(mids.chars().map(ch));
        r.push(tail);
        r
    }

    match mode {
        TextLower => vec![
            text_row(
                ctrl("123", Mode(Special), 3),
                "qwertyuiop",
                ctrl("⌫", Backspace, 3),
            ),
            text_row(
                ctrl("ABC", Mode(TextUpper), 3),
                "asdfghjkl",
                ctrl("↵", Newline, 3),
            ),
            "_-zxcvbnm.,:".chars().map(ch).collect(),
            bottom_row(),
        ],
        TextUpper => vec![
            text_row(
                ctrl("123", Mode(Special), 3),
                "QWERTYUIOP",
                ctrl("⌫", Backspace, 3),
            ),
            text_row(
                ctrl("abc", Mode(TextLower), 3),
                "ASDFGHJKL",
                ctrl("↵", Newline, 3),
            ),
            "_-ZXCVBNM.,:".chars().map(ch).collect(),
            bottom_row(),
        ],
        Special => vec![
            {
                let mut r: Vec<Key> = "0123456789".chars().map(ch).collect();
                r.push(ctrl("⌫", Backspace, 3));
                r
            },
            text_row(ctrl("abc", Mode(TextLower), 3), "+-/*=%!?#<>", ctrl("↵", Newline, 3)),
            "\\@$(){}[];\"'".chars().map(ch).collect(),
            bottom_row(),
        ],
        Number => vec![
            {
                let mut r: Vec<Key> = "123".chars().map(ch).collect();
                r.push(ctrl("▼", KeyAction::Cancel, 1));
                r
            },
            {
                let mut r: Vec<Key> = "456".chars().map(ch).collect();
                r.push(ctrl("OK", Ready, 1));
                r
            },
            {
                let mut r: Vec<Key> = "789".chars().map(ch).collect();
                r.push(ctrl("⌫", Backspace, 1));
                r
            },
            vec![
                ctrl("abc", Mode(TextLower), 1),
                ch('0'),
                ch('.'),
                ctrl("←", CursorLeft, 1),
                ctrl("→", CursorRight, 1),
            ],
        ],
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Keyboard<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
        let h = self
            .height
            .resolve(constraints.max.height, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.bounds = rect;
        let kb_bounds = self.key_bounds_for(rect);
        self.keys.arrange(kb_bounds);
    }

    fn rect(&self) -> Rectangle {
        self.bounds
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        if let Some(btn) = self.show_button_rect() {
            let br = btn.top_left + Point::new(btn.size.width as i32, btn.size.height as i32);
            let inside = point.x >= btn.top_left.x
                && point.x < br.x
                && point.y >= btn.top_left.y
                && point.y < br.y;
            if inside {
                // Toggle reveal on release; swallow press/move so the tap on
                // the field strip never reaches the keys below.
                return (phase == TouchPhase::Up).then(|| self.toggle_reveal.clone());
            }
        }
        self.keys.handle_touch(point, phase)
    }

    fn mark_pressed(&mut self, point: Point) {
        self.keys.mark_pressed(point);
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        renderer.fill_rect(self.bounds, theme.background.base)?;

        if self.show_field {
            let x0 = self.bounds.top_left.x;
            let y0 = self.bounds.top_left.y;
            let width = self.bounds.size.width;

            // Title
            renderer.draw_text(
                &self.title,
                Point::new(x0 + (width / 2) as i32, y0 + 18),
                theme.default_font(),
                theme.background.on_base,
                Alignment::Center,
            )?;

            // Input field outline — narrowed to leave room for the password
            // Show/Hide button when present.
            let field_w = if self.is_password {
                width.saturating_sub(16 + SHOW_W + 4)
            } else {
                width.saturating_sub(16)
            };
            let field = Rectangle::new(Point::new(x0 + 8, y0 + 30), Size::new(field_w, 28));
            renderer.stroke_rect(field, theme.button.border)?;

            // Masked unless the user tapped Show.
            let display_text = if self.is_password && !self.reveal {
                "*".repeat(self.input.chars().count())
            } else {
                self.input.clone()
            };
            renderer.draw_text(
                &display_text,
                Point::new(x0 + 14, y0 + 49),
                theme.default_font(),
                theme.background.on_base,
                Alignment::Left,
            )?;

            // Password Show/Hide toggle.
            if let Some(btn) = self.show_button_rect() {
                renderer.fill_rect(btn, theme.button.base)?;
                renderer.stroke_rect(btn, theme.button.border)?;
                let label = if self.reveal { "Hide" } else { "Show" };
                let glyph_h = theme.default_font().character_size.height as i32;
                renderer.draw_text(
                    label,
                    Point::new(
                        btn.top_left.x + btn.size.width as i32 / 2,
                        btn.top_left.y + btn.size.height as i32 / 2 + glyph_h / 3,
                    ),
                    theme.default_font(),
                    theme.button.on_base,
                    Alignment::Center,
                )?;
            }
        }

        // Key grid.
        self.keys.draw(renderer, theme)
    }
}
