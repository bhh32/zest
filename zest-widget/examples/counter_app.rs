//! Counter with conditional auto-tick Subscription.

extern crate alloc;
use alloc::string::String;
use core::fmt::Write as _;
use core::hash::Hash;
use embassy_time::Duration;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone, Hash)]
enum Msg {
    Add(i32),
    Reset,
    ToggleAuto,
    Tick,
}

struct CounterScreen {
    count: i32,
    auto: bool,
    theme: Theme<'static, Rgb565>,
}

impl ScreenView<Rgb565, Msg> for CounterScreen {
    fn name(&self) -> &'static str {
        "Counter"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let mut count_str = String::new();
        let _ = write!(&mut count_str, "{}", self.count);
        let auto_label = if self.auto { "Auto: ON" } else { "Auto: OFF" };

        let display = Container::new().height(Length::Fill).child(
            Text::new(count_str)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .font(self.theme.typography.heading)
                .color(self.theme.background.on_base),
        );

        let buttons = Row::new()
            .spacing(4)
            .push(Button::new("-10").on_press(Msg::Add(-10)))
            .push(Button::new("-1").on_press(Msg::Add(-1)))
            .push(
                Button::new("+1")
                    .on_press(Msg::Add(1))
                    .class(ButtonClass::Suggested),
            )
            .push(
                Button::new("+10")
                    .on_press(Msg::Add(10))
                    .class(ButtonClass::Suggested),
            )
            .push(
                Button::new("Reset")
                    .on_press(Msg::Reset)
                    .class(ButtonClass::Destructive),
            );

        let auto = Button::new(auto_label)
            .on_press(Msg::ToggleAuto)
            .class(if self.auto {
                ButtonClass::Success
            } else {
                ButtonClass::Standard
            });

        Column::new()
            .spacing(6)
            .push(display)
            .push(buttons)
            .push(auto)
            .into_element()
    }
}

struct App {
    screen: CounterScreen,
}

impl Application for App {
    type Message = Msg;
    type Color = Rgb565;
    type Screen = CounterScreen;

    fn init() -> (Self, Task<Msg>) {
        let app = Self {
            screen: CounterScreen {
                count: 0,
                auto: false,
                theme: convert_theme(&dark::THEME),
            },
        };
        (app, Task::none())
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let s = &mut self.screen;
        match m {
            Msg::Add(n) => s.count = s.count.saturating_add(n),
            Msg::Reset => s.count = 0,
            Msg::ToggleAuto => s.auto = !s.auto,
            Msg::Tick => s.count = s.count.saturating_add(1),
        }
        Task::none()
    }

    fn view(&self) -> &CounterScreen {
        &self.screen
    }

    fn subscription(&self) -> Subscription<Msg> {
        if self.screen.auto {
            zest::time::every(Duration::from_secs(1), Msg::Tick)
        } else {
            Subscription::none()
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Counter").await;
}
