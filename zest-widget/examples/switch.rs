//! Switch demo: two toggle switches with a live status line.
//!
//! Immediate-mode: the host owns each `bool` and flips it in `update`
//! when a switch's `on_toggle` fires.

extern crate alloc;
use alloc::string::{String, ToString};
use core::fmt::Write as _;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const LIGHTS_ID: WidgetId = WidgetId::new(0x630);
const FAN_ID: WidgetId = WidgetId::new(0x631);

#[derive(Clone)]
enum Msg {
    SetLights(bool),
    SetFan(bool),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    lights: bool,
    fan: bool,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            lights: true,
            fan: false,
        }
    }

    fn labelled(
        &self,
        text: &'static str,
        switch: Switch<'static, Rgb565, Msg>,
    ) -> Row<'static, Rgb565, Msg> {
        Row::new()
            .spacing(10)
            .push(
                Text::new(text.to_string())
                    .align_y(Vertical::Center)
                    .color(self.theme.background.on_base),
            )
            .push(switch.width(Length::Fixed(48)))
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Switch"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let mut status = String::new();
        let _ = write!(
            &mut status,
            "lights: {}   fan: {}",
            if self.lights { "on" } else { "off" },
            if self.fan { "on" } else { "off" }
        );

        Column::new()
            .spacing(12)
            .push(
                Text::new("Switches".to_string())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                self.labelled(
                    "Lights",
                    Switch::new(self.lights)
                        .id(LIGHTS_ID)
                        .on_toggle(Msg::SetLights),
                )
                .height(Length::Fixed(28)),
            )
            .push(
                self.labelled(
                    "Fan",
                    Switch::new(self.fan).id(FAN_ID).on_toggle(Msg::SetFan),
                )
                .height(Length::Fixed(28)),
            )
            .push(horizontal_divider())
            .push(
                Text::new(status)
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

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let s = &mut self.screen;
        match m {
            Msg::SetLights(v) => s.lights = v,
            Msg::SetFan(v) => s.fan = v,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Switch").await;
}
