//! On-screen QWERTY keyboard. Generic over color *and* the user's
//! message type — a callback at construction time translates each
//! [`KeyAction`] into a user message that the screen interprets.
//!
//! In the transient widget model, the screen owns the keyboard's
//! state (input string, layout, shift flag). Each frame `view()`
//! constructs a fresh `Keyboard` from current state. The screen
//! handles `KeyAction` messages in its `update` to mutate state.

use super::{Widget, button::Button, column::Column, row::Row};
use alloc::string::{String, ToString};
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// A key action emitted by the keyboard. The screen receives these as
/// messages (via a translation callback passed at construction) and
/// mutates its own state in `update`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    /// User tapped a character key.
    Char(char),
    /// User tapped backspace.
    Backspace,
    /// User tapped shift.
    Shift,
    /// User tapped the alpha/numeric layout toggle.
    SwitchLayout,
    /// User tapped space.
    Space,
    /// User tapped Done / OK.
    Done,
    /// User tapped Cancel.
    Cancel,
}

/// Keyboard layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// QWERTY alphabetic layout.
    Alpha,
    /// Numeric and symbol layout.
    Numeric,
}

/// On-screen QWERTY keyboard.
///
/// Constructed fresh each frame from screen state. The screen owns
/// the input text, current layout, and shift flag; it passes them in
/// at construction. Each button emits a user `M` value computed at
/// build time via the `on_action` callback.
pub struct Keyboard<'a, C: PixelColor, M: Clone> {
    bounds: Rectangle,
    title: String,
    input: String,
    is_password: bool,
    keys: Column<'a, C, M>,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Keyboard<'a, C, M> {
    /// Construct a keyboard. `on_action` translates a `KeyAction` into
    /// the user's message type. Position and size are assigned by the
    /// parent via `arrange`.
    pub fn new<F>(
        title: impl Into<String>,
        input: impl Into<String>,
        is_password: bool,
        layout: Layout,
        shift: bool,
        on_action: F,
    ) -> Self
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let title = title.into();
        let input = input.into();
        let keys = Self::build_keys(layout, shift, on_action);
        Self {
            bounds: Rectangle::zero(),
            title,
            input,
            is_password,
            keys,
            width: Length::Fill,
            height: Length::Fill,
        }
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

    fn key_bounds_for(bounds: Rectangle) -> Rectangle {
        // Top 70px reserved for title + input field.
        Rectangle::new(
            Point::new(bounds.top_left.x, bounds.top_left.y + 70),
            Size::new(bounds.size.width, bounds.size.height.saturating_sub(70)),
        )
    }

    fn build_keys<F>(layout: Layout, shift: bool, on_action: F) -> Column<'a, C, M>
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        Column::new()
            .push(Self::build_letter_row(layout, shift, 0, on_action))
            .push(Self::build_letter_row(layout, shift, 1, on_action))
            .push(Self::build_row2(layout, shift, on_action))
            .push(Self::build_bottom_row(layout, on_action))
    }

    fn build_letter_row<F>(
        layout: Layout,
        shift: bool,
        row: usize,
        on_action: F,
    ) -> Row<'a, C, M>
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let chars: &[char] = match (layout, row) {
            (Layout::Alpha, 0) => &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
            (Layout::Alpha, 1) => &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
            (Layout::Numeric, 0) => &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
            (Layout::Numeric, 1) => &['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'],
            _ => &[],
        };

        let mut row_widget = Row::new();
        for &ch in chars {
            let display_ch = if shift && layout == Layout::Alpha && ch.is_ascii_lowercase() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            let mut buf = [0u8; 4];
            let label = display_ch.encode_utf8(&mut buf).to_string();
            let btn = Button::new(label)
                .on_press(on_action(KeyAction::Char(ch)));
            row_widget = row_widget.push(btn);
        }

        if matches!((layout, row), (Layout::Alpha, 1)) {
            let btn = Button::new("<-")
                .on_press(on_action(KeyAction::Backspace));
            row_widget = row_widget.push(btn);
        }

        row_widget
    }

    fn build_row2<F>(layout: Layout, shift: bool, on_action: F) -> Row<'a, C, M>
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let mut row = Row::new();
        match layout {
            Layout::Alpha => {
                let shift_label = if shift { "^" } else { "v" };
                row = row.push(
                    Button::new(shift_label)
                        .on_press(on_action(KeyAction::Shift)),
                );

                for ch in ['z', 'x', 'c', 'v', 'b', 'n', 'm', '.'] {
                    let dch = if shift && ch.is_ascii_lowercase() {
                        ch.to_ascii_uppercase()
                    } else {
                        ch
                    };
                    let mut buf = [0u8; 4];
                    let label = dch.encode_utf8(&mut buf).to_string();
                    row = row.push(
                        Button::new(label)
                            .on_press(on_action(KeyAction::Char(ch))),
                    );
                }
                row = row.push(
                    Button::new("OK").on_press(on_action(KeyAction::Done)),
                );
            }
            Layout::Numeric => {
                for ch in ['-', '_', '+', '=', '/', '\\', ';', ':', ','] {
                    let mut buf = [0u8; 4];
                    let label = ch.encode_utf8(&mut buf).to_string();
                    row = row.push(
                        Button::new(label)
                            .on_press(on_action(KeyAction::Char(ch))),
                    );
                }
                row = row.push(
                    Button::new("OK").on_press(on_action(KeyAction::Done)),
                );
            }
        }
        row
    }

    /// Bottom row uses weighted children: toggle(2) + space(6) + cancel(2).
    fn build_bottom_row<F>(layout: Layout, on_action: F) -> Row<'a, C, M>
    where
        F: Fn(KeyAction) -> M + Copy + 'a,
    {
        let toggle_label = match layout {
            Layout::Alpha => "123",
            Layout::Numeric => "abc",
        };

        Row::new()
            .push(
                Button::new(toggle_label)
                    .on_press(on_action(KeyAction::SwitchLayout))
                    .width(Length::FillPortion(2)),
            )
            .push(
                Button::new("space")
                    .on_press(on_action(KeyAction::Space))
                    .width(Length::FillPortion(6)),
            )
            .push(
                Button::new("X")
                    .on_press(on_action(KeyAction::Cancel))
                    .width(Length::FillPortion(2)),
            )
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
        let kb_bounds = Self::key_bounds_for(rect);
        self.keys.arrange(kb_bounds);
    }

    fn rect(&self) -> Rectangle {
        self.bounds
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
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

        // Input field outline.
        let field = Rectangle::new(
            Point::new(x0 + 8, y0 + 30),
            Size::new(width.saturating_sub(16), 28),
        );
        renderer.stroke_rect(field, theme.button.border)?;

        let display_text = if self.is_password {
            "*".repeat(self.input.len())
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

        // Key grid.
        self.keys.draw(renderer, theme)
    }
}
