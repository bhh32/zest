//! Spin button demo: horizontal and vertical steppers with live readout.
//!
//! The host owns the numeric state. Tab moves through the minus/plus controls,
//! and Enter activates the focused side.

extern crate alloc;
use alloc::format;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const HOUR_ID: WidgetId = WidgetId::new(0x650);
const COUNT_ID: WidgetId = WidgetId::new(0x660);

#[derive(Clone)]
enum Msg {
    SetHour(i32),
    SetCount(i32),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    hour: i32,
    count: i32,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            hour: 9,
            count: 3,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "SpinButton"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        Column::new()
            .spacing(10)
            .push(
                Text::new("Spin buttons")
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                horizontal_spin_button(self.hour)
                    .id(HOUR_ID)
                    .min(0)
                    .max(23)
                    .display(format!("{:02}:00", self.hour))
                    .on_change(Msg::SetHour)
                    .height(Length::Fixed(28)),
            )
            .push(
                Row::new()
                    .spacing(8)
                    .push(
                        vertical_spin_button(self.count)
                            .id(COUNT_ID)
                            .min(0)
                            .max(9)
                            .on_change(Msg::SetCount)
                            .width(Length::Fixed(28))
                            .height(Length::Fixed(84)),
                    )
                    .push(
                        Text::new(format!("Counter: {}", self.count))
                            .align_y(Vertical::Center)
                            .color(self.theme.background.on_base),
                    ),
            )
            .push(horizontal_divider())
            .push(
                Text::new(format!("Hour: {:02}:00   Count: {}", self.hour, self.count))
                    .font(self.theme.typography.caption)
                    .color(self.theme.palette.neutral_2),
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

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::SetHour(value) => self.screen.hour = value,
            Msg::SetCount(value) => self.screen.count = value,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - SpinButton").await;
}
