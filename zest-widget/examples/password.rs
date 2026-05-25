//! Password entry with a Show/Hide reveal toggle.
//!
//! The keyboard masks the field as `*` by default; tapping the **Show** button
//! in the preview field emits [`KeyAction::ToggleReveal`], which the host flips
//! and passes back via `.reveal(..)` so the user can verify what the small keys
//! typed. Tap OK to "sign in".

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Key(KeyAction),
    Restart,
}

enum State {
    Entering {
        password: String,
        mode: KeyboardMode,
        reveal: bool,
    },
    Done,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    state: State,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            state: State::Entering {
                password: String::new(),
                mode: KeyboardMode::TextLower,
                reveal: false,
            },
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Password"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        match &self.state {
            State::Entering {
                password,
                mode,
                reveal,
            } => Keyboard::new(*mode, Msg::Key)
                .title("Enter password")
                .input(password.clone())
                .is_password(true)
                .reveal(*reveal)
                .show_field(true)
                .into_element(),
            State::Done => Column::new()
                .spacing(8)
                .push(vertical_space())
                .push(
                    Text::new("Signed in")
                        .align_x(Horizontal::Center)
                        .font(self.theme.typography.heading)
                        .color(self.theme.background.on_base),
                )
                .push(vertical_space())
                .push(
                    Button::new("Again")
                        .on_press(Msg::Restart)
                        .class(ButtonClass::Suggested)
                        .width(Length::Shrink),
                )
                .into_element(),
        }
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
        (
            Self {
                screen: Screen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let s = &mut self.screen;
        match m {
            Msg::Restart => {
                s.state = State::Entering {
                    password: String::new(),
                    mode: KeyboardMode::TextLower,
                    reveal: false,
                }
            }
            Msg::Key(action) => {
                let State::Entering {
                    password,
                    mode,
                    reveal,
                } = &mut s.state
                else {
                    return Task::none();
                };
                match action {
                    KeyAction::Char(c) => password.push(c),
                    KeyAction::Backspace => {
                        password.pop();
                    }
                    KeyAction::Mode(m) => *mode = m,
                    KeyAction::ToggleReveal => *reveal = !*reveal,
                    KeyAction::Ready | KeyAction::Newline => {
                        if !password.is_empty() {
                            s.state = State::Done;
                        }
                    }
                    KeyAction::Cancel => password.clear(),
                    KeyAction::CursorLeft | KeyAction::CursorRight => {}
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Password").await;
}
