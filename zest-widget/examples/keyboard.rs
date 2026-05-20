//! Name greeter — two-state demo of Keyboard input + Done/Cancel handling.

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const MAX_LEN: usize = 32;

#[derive(Clone)]
enum Msg {
    Key(KeyAction),
    Restart,
}

enum State {
    Asking { input: String, layout: Layout, shift: bool },
    Greeting { name: String },
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    state: State,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            state: State::Asking {
                input: String::new(),
                layout: Layout::Alpha,
                shift: true,
            },
        }
    }

    fn reset(&mut self) {
        self.state = State::Asking {
            input: String::new(),
            layout: Layout::Alpha,
            shift: true,
        };
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Keyboard"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        match &self.state {
            State::Asking { input, layout, shift } => Keyboard::new(
                "What's your name?",
                input.clone(),
                false,
                *layout,
                *shift,
                Msg::Key,
            )
            .into_element(),
            State::Greeting { name } => {
                let greeting = alloc::format!("Hello, {name}!");
                Column::new()
                    .spacing(8)
                    .push(vertical_space())
                    .push(
                        Text::new(greeting)
                            .align_x(Horizontal::Center)
                            .font(self.theme.typography.heading)
                            .color(self.theme.background.on_base),
                    )
                    .push(vertical_space())
                    .push(
                        Button::new("Start over")
                            .on_press(Msg::Restart)
                            .class(ButtonClass::Suggested)
                            .width(Length::Shrink),
                    )
                    .into_element()
            }
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
        (Self { screen: Screen::new() }, Task::none())
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let s = &mut self.screen;
        match m {
            Msg::Restart => {
                s.reset();
                Task::none()
            }
            Msg::Key(action) => {
                let State::Asking { input, layout, shift } = &mut s.state else {
                    return Task::none();
                };
                match action {
                    KeyAction::Char(ch) => {
                        if input.len() < MAX_LEN {
                            let to_push = if *layout == Layout::Alpha
                                && *shift
                                && ch.is_ascii_lowercase()
                            {
                                *shift = false;
                                ch.to_ascii_uppercase()
                            } else {
                                ch
                            };
                            input.push(to_push);
                        }
                    }
                    KeyAction::Backspace => {
                        input.pop();
                    }
                    KeyAction::Shift => *shift = !*shift,
                    KeyAction::SwitchLayout => {
                        *layout = match *layout {
                            Layout::Alpha => Layout::Numeric,
                            Layout::Numeric => Layout::Alpha,
                        };
                        *shift = false;
                    }
                    KeyAction::Space => {
                        if input.len() < MAX_LEN {
                            input.push(' ');
                        }
                    }
                    KeyAction::Done => {
                        let name = core::mem::take(input);
                        if !name.is_empty() {
                            s.state = State::Greeting { name };
                        }
                    }
                    KeyAction::Cancel => s.reset(),
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Keyboard").await;
}
