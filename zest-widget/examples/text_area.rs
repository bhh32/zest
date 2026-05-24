//! TextArea editor — live multi-line editing wired to an on-screen Keyboard.
//!
//! The host owns `text: String` and `cursor: usize` (a char index). The
//! [`TextArea`] renders the text + cursor on the top half and maps taps back
//! to a char index; the [`Keyboard`] on the bottom half drives edits via
//! [`KeyAction`]. `update` performs the actual string mutations:
//!
//! * `Char(c)` — insert at the cursor, then advance it (uppercase is its
//!   own keymap, so the char is already cased).
//! * `Backspace` — delete the char before the cursor; `Newline` — insert `\n`.
//! * `CursorLeft` / `CursorRight` — move the caret; a TextArea tap also
//!   repositions it. `Mode(m)` — switch keymap (1#, ABC/abc).
//! * `Ready` (OK) — clear the buffer; `Cancel` (hide) — no-op.
//!
//! Type past the bottom of the box to watch it char-wrap and auto-scroll so
//! the caret stays visible.

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

/// Maximum buffer length, to keep the demo bounded.
const MAX_LEN: usize = 512;

#[derive(Clone)]
enum Msg {
    /// A key from the on-screen keyboard.
    Key(KeyAction),
    /// A tap inside the TextArea repositioned the cursor (char index).
    Move(usize),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    text: String,
    /// Cursor position as a char index into `text`.
    cursor: usize,
    /// Active keyboard keymap (lower/upper/special/number).
    kb_mode: KeyboardMode,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            text: String::from("Type below.\nTap to move the cursor."),
            cursor: 0,
            kb_mode: KeyboardMode::TextLower,
        }
    }

    /// Byte offset of the cursor (clamped to a char boundary).
    fn cursor_byte(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map_or(self.text.len(), |(b, _)| b)
    }

    /// Insert a char at the cursor and advance it.
    fn insert(&mut self, ch: char) {
        if self.text.chars().count() >= MAX_LEN {
            return;
        }
        let at = self.cursor_byte();
        self.text.insert(at, ch);
        self.cursor += 1;
    }

    /// Delete the char before the cursor and retreat it.
    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        // Byte range of the char immediately before the cursor.
        let prev = self.cursor - 1;
        let start = self
            .text
            .char_indices()
            .nth(prev)
            .map_or(0, |(b, _)| b);
        let end = self.cursor_byte();
        self.text.replace_range(start..end, "");
        self.cursor = prev;
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "TextArea"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let editor = TextArea::new(self.text.clone())
            .cursor(self.cursor)
            .font(self.theme.typography.body)
            .placeholder("Start typing…")
            .color(self.theme.background.on_base)
            .cursor_color(self.theme.accent.base)
            .on_tap(Msg::Move)
            .width(Length::Fill)
            .height(Length::Fill);

        // Keys only: the TextArea above is the text display, so the
        // keyboard's built-in preview field stays off (the default).
        let keyboard = Keyboard::new(self.kb_mode, Msg::Key)
            .width(Length::Fill)
            .height(Length::Fill);

        Column::new()
            .spacing(4)
            // Editor on top, keyboard below, split 2:3.
            .push(
                Container::new()
                    .child(editor)
                    .padding(6)
                    .width(Length::Fill)
                    .height(Length::FillPortion(2)),
            )
            .push(
                Container::new()
                    .child(keyboard)
                    .width(Length::Fill)
                    .height(Length::FillPortion(3)),
            )
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Msg;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Msg>) {
        (Self { screen: Screen::new() }, Task::none())
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let s = &mut self.screen;
        match m {
            Msg::Move(index) => {
                s.cursor = index.min(s.text.chars().count());
            }
            Msg::Key(action) => match action {
                // Uppercase is its own keymap, so the char arrives already
                // cased — no host-side shift handling needed.
                KeyAction::Char(ch) => s.insert(ch),
                KeyAction::Backspace => s.backspace(),
                KeyAction::Newline => s.insert('\n'),
                KeyAction::CursorLeft => s.cursor = s.cursor.saturating_sub(1),
                KeyAction::CursorRight => {
                    s.cursor = (s.cursor + 1).min(s.text.chars().count());
                }
                KeyAction::Mode(mode) => s.kb_mode = mode,
                KeyAction::Ready => {
                    s.text.clear();
                    s.cursor = 0;
                }
                KeyAction::Cancel => {}
            },
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - TextArea").await;
}
