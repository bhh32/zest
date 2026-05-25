//! Window: a titled panel — a title bar (title text + optional close button)
//! above a content area holding one child.
//!
//! The close button (present only when [`Window::on_close`] is set) emits a
//! host message on release. Here the host counts close taps and shows the
//! count inside the window's content child.

extern crate alloc;
use alloc::format;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Close,
    Bump,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    closes: u32,
    bumps: u32,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            closes: 0,
            bumps: 0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Window"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let body = Column::new()
            .spacing(8)
            .push(
                Text::new(format!("Bumps: {}   Closes: {}", self.bumps, self.closes))
                    .align_x(Horizontal::Center)
                    .color(self.theme.background.on_base),
            )
            .push(
                Button::new("Bump")
                    .class(ButtonClass::Suggested)
                    .on_press(Msg::Bump),
            );

        let window = Window::new()
            .title("Demo Window")
            .on_close(Msg::Close)
            .padding(10)
            .width(Length::Fixed(260))
            .height(Length::Fixed(180))
            .child(body);

        // Center the fixed-size window inside the screen.
        Stack::new()
            .push_aligned(window, Horizontal::Center, Vertical::Center)
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
        (
            Self {
                screen: Screen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Close => self.screen.closes += 1,
            Msg::Bump => self.screen.bumps += 1,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Window").await;
}
