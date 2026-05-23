//! LED indicators: a row of status lights in on/off states and varying
//! colors, with a button that toggles the whole bank.

extern crate alloc;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Toggle,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    powered: bool,
}

impl Screen {
    fn light(&self, on: bool, color: Rgb565, label: &'static str) -> Column<'static, Rgb565, Msg> {
        Column::new()
            .spacing(4)
            .push(
                LED::from_state(on, color)
                    .width(Length::Fixed(28))
                    .height(Length::Fixed(28)),
            )
            .push(
                Text::new(label)
                    .align_x(Horizontal::Center)
                    .height(Length::Fixed(16))
                    .color(self.theme.background.on_base),
            )
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "LED"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let on = self.powered;
        let bank = Row::new()
            .spacing(16)
            .push(self.light(on, self.theme.success.base, "OK"))
            .push(self.light(on, self.theme.warning.base, "WARN"))
            .push(self.light(on, self.theme.destructive.base, "ERR"))
            .push(self.light(true, self.theme.accent.base, "PWR"));

        Column::new()
            .spacing(8)
            .push(
                Text::new("LED status bank")
                    .align_x(Horizontal::Center)
                    .height(Length::Fixed(20))
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(bank)
            .push(vertical_space())
            .push(
                Button::new(if on { "Power off" } else { "Power on" })
                    .on_press(Msg::Toggle)
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
                screen: Screen {
                    theme: convert_theme(&dark::THEME),
                    powered: true,
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Toggle => self.screen.powered = !self.screen.powered,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - LED").await;
}
