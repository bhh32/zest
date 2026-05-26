//! Progress bar demo: an animated download that advances on a timer, plus
//! a reset button.
//!
//! `ProgressBar` is passive — it just renders the host-owned value. Here a
//! `time::every` subscription nudges the value forward each tick.

extern crate alloc;
use alloc::string::{String, ToString};
use core::fmt::Write as _;
use embassy_time::Duration;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone, Hash)]
enum Msg {
    Tick,
    Reset,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    progress: f32,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            progress: 0.0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "ProgressBar"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let mut label = String::new();
        let _ = write!(
            &mut label,
            "downloading… {}%",
            (self.progress * 100.0) as i32
        );

        Column::new()
            .spacing(12)
            .push(
                Text::new("Progress".to_string())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                Text::new(label)
                    .font(self.theme.typography.caption)
                    .color(self.theme.palette.neutral_2),
            )
            .push(
                ProgressBar::new(self.progress)
                    .range(0.0, 1.0)
                    .height(Length::Fixed(14)),
            )
            .push(
                ProgressBar::new(self.progress)
                    .range(0.0, 1.0)
                    .color(self.theme.success.base)
                    .height(Length::Fixed(8)),
            )
            .push(horizontal_divider())
            .push(
                Button::new("Reset")
                    .on_press(Msg::Reset)
                    .class(ButtonClass::Standard)
                    .height(Length::Fixed(32)),
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
            Msg::Tick => {
                s.progress += 0.02;
                if s.progress > 1.0 {
                    s.progress = 1.0;
                }
            }
            Msg::Reset => s.progress = 0.0,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }

    fn subscription(&self) -> Subscription<Msg> {
        if self.screen.progress < 1.0 {
            zest::time::every(Duration::from_millis(120), Msg::Tick)
        } else {
            Subscription::none()
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - ProgressBar").await;
}
