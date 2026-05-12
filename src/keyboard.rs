//! On-screen QWERTY keyboard. Self-contained; doesn't compose with framework
//! containers because layout is hand-tuned.

use crate::{
    IntoElement, Renderer, Theme, Widget, button::Button, column::Column, renderer::RenderError,
    row::Row,
};
use alloc::string::{String, ToString};
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};

#[derive(Clone)]
enum KeyAction {
    Char(char),
    Backspace,
    Shift,
    SwitchLayout,
    Space,
    Done,
    Cancel,
}

pub enum KeyboardEvent {
    Continue,
    Done(String),
    Cancel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    Alpha,
    Numeric,
}

/// QWERTY keyboard, Generic over color so it inherits from the app's theme.
pub struct Keyboard<'a, C: PixelColor> {
    bounds: Rectangle,
    title: String,
    input: String,
    is_password: bool,
    layout: Layout,
    shift: bool,
    max_len: usize,
    keys: Column<'a, C, KeyAction>,
}

impl<'a, C: PixelColor + 'a> Keyboard<'a, C> {
    pub fn new(
        bounds: Rectangle,
        title: impl Into<String>,
        initial: String,
        is_password: bool,
    ) -> Self {
        let mut kb = Self {
            bounds,
            title: title.into(),
            input: initial,
            is_password,
            layout: Layout::Alpha,
            shift: false,
            max_len: 63,
            keys: Self::build_keys(bounds, Layout::Alpha, false),
        };
        kb.keys = Self::build_keys(kb.key_bounds(), kb.layout, kb.shift);
        kb.keys.set_rect(kb.key_bounds());
        kb
    }

    /// Ther vertical region available for the keyboard's 4 rows.
    fn key_bounds(&self) -> Rectangle {
        // Top 70px reserved for title + input field.
        Rectangle::new(
            Point::new(self.bounds.top_left.x, self.bounds.top_left.y + 70),
            Size::new(
                self.bounds.size.width,
                self.bounds.size.height.saturating_sub(70),
            ),
        )
    }

    /// Construct the 4-row Column<Row<Button>> tree for the current layout.
    fn build_keys(bounds: Rectangle, layout: Layout, shift: bool) -> Column<'a, C, KeyAction> {
        let row0 = Self::build_letter_row(layout, shift, 0).into_element();
        let row1 = Self::build_letter_row(layout, shift, 1).into_element();
        let row2 = Self::build_row2(layout, shift).into_element();
        let row3 = Self::build_bottom_row(layout).into_element();

        Column::new(bounds)
            .push(row0)
            .push(row1)
            .push(row2)
            .push(row3)
    }

    fn build_letter_row(layout: Layout, shift: bool, row: usize) -> Row<'a, C, KeyAction> {
        let chars: &[char] = match (layout, row) {
            (Layout::Alpha, 0) => &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
            (Layout::Alpha, 1) => &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
            (Layout::Numeric, 0) => &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
            (Layout::Numeric, 1) => &['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'],
            _ => &[],
        };

        let mut row_widget = Row::new(Rectangle::zero());
        for &ch in chars {
            let display_ch = if shift && layout == Layout::Alpha && ch.is_ascii_lowercase() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            let mut buf = [0u8; 4];
            let label = display_ch.encode_utf8(&mut buf).to_string();
            let btn = Button::new(Rectangle::zero(), label)
                .on_press(KeyAction::Char(ch))
                .into_element();
            row_widget = row_widget.push(btn);
        }

        // Row 1 in alpha layout has 9 letters then backspace.
        if matches!((layout, row), (Layout::Alpha, 1)) {
            let btn = Button::new(Rectangle::zero(), "<-")
                .on_press(KeyAction::Backspace)
                .into_element();
            row_widget = row_widget.push(btn);
        }

        row_widget
    }

    fn build_row2(layout: Layout, shift: bool) -> Row<'a, C, KeyAction> {
        let mut row = Row::new(Rectangle::zero());

        match layout {
            Layout::Alpha => {
                let shift_label = if shift { "^" } else { "v" };
                row = row.push(
                    Button::new(Rectangle::zero(), shift_label)
                        .on_press(KeyAction::Shift)
                        .into_element(),
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
                        Button::new(Rectangle::zero(), label)
                            .on_press(KeyAction::Char(ch))
                            .into_element(),
                    );
                }
                row = row.push(
                    Button::new(Rectangle::zero(), "OK")
                        .on_press(KeyAction::Done)
                        .into_element(),
                );
            }
            Layout::Numeric => {
                for ch in ['-', '_', '+', '=', '/', '\\', ';', ':', ','] {
                    let mut buf = [0u8; 4];
                    let label = ch.encode_utf8(&mut buf).to_string();
                    row = row.push(
                        Button::new(Rectangle::zero(), label)
                            .on_press(KeyAction::Char(ch))
                            .into_element(),
                    );
                }
                row = row.push(
                    Button::new(Rectangle::zero(), "OK")
                        .on_press(KeyAction::Done)
                        .into_element(),
                );
            }
        }
        row
    }

    /// Bottom row uses weighted children: toggle(2) + space(6) + cancel(2).
    fn build_bottom_row(layout: Layout) -> Row<'a, C, KeyAction> {
        let toggle_label = match layout {
            Layout::Alpha => "123",
            Layout::Numeric => "abc",
        };

        Row::new(Rectangle::zero())
            .push_weighted(
                Button::new(Rectangle::zero(), toggle_label)
                    .on_press(KeyAction::SwitchLayout)
                    .into_element(),
                2,
            )
            .push_weighted(
                Button::new(Rectangle::zero(), "space")
                    .on_press(KeyAction::Space)
                    .into_element(),
                6,
            )
            .push_weighted(
                Button::new(Rectangle::zero(), "X")
                    .on_press(KeyAction::Cancel)
                    .into_element(),
                2,
            )
    }

    /// Apply a key action. Returns the public KeyboardEvent.
    fn apply(&mut self, action: KeyAction) -> KeyboardEvent {
        match action {
            KeyAction::Char(ch) => {
                if self.input.len() < self.max_len {
                    let to_push =
                        if self.layout == Layout::Alpha && self.shift && ch.is_ascii_lowercase() {
                            self.shift = false;
                            self.rebuild();
                            ch.to_ascii_uppercase()
                        } else {
                            ch
                        };
                    self.input.push(to_push);
                }
                KeyboardEvent::Continue
            }
            KeyAction::Backspace => {
                self.input.pop();
                KeyboardEvent::Continue
            }
            KeyAction::Shift => {
                self.shift = !self.shift;
                self.rebuild();
                KeyboardEvent::Continue
            }
            KeyAction::SwitchLayout => {
                self.layout = match self.layout {
                    Layout::Alpha => Layout::Numeric,
                    Layout::Numeric => Layout::Alpha,
                };
                self.shift = false;
                self.rebuild();
                KeyboardEvent::Continue
            }
            KeyAction::Space => {
                if self.input.len() < self.max_len {
                    self.input.push(' ');
                }
                KeyboardEvent::Continue
            }
            KeyAction::Done => KeyboardEvent::Done(core::mem::take(&mut self.input)),
            KeyAction::Cancel => KeyboardEvent::Cancel,
        }
    }

    fn rebuild(&mut self) {
        let bounds = self.key_bounds();
        self.keys = Self::build_keys(bounds, self.layout, self.shift);
        self.keys.set_rect(bounds);
    }

    pub fn handle_touch(&mut self, point: Point) -> KeyboardEvent {
        let result = if let Some(action) = self.keys.handle_touch(point) {
            self.apply(action)
        } else {
            KeyboardEvent::Continue
        };

        let _ = self.keys.handle_touch(Point::new(-1, -1));

        result
    }

    pub fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError>
    where
        't: 'a,
    {
        renderer.fill_rect(self.bounds, theme.background)?;

        let x0 = self.bounds.top_left.x;
        let y0 = self.bounds.top_left.y;
        let width = self.bounds.size.width;

        // Title
        renderer.draw_text(
            &self.title,
            Point::new(x0 + (width / 2) as i32, y0 + 18),
            theme.label.font,
            theme.label.color,
            Alignment::Center,
        )?;

        // Input field.
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
            theme.label.font,
            theme.label.color,
            Alignment::Left,
        )?;

        // Key grid.
        self.keys.draw(renderer, theme)
    }
}
