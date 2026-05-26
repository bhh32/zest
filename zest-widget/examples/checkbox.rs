//! Checkbox demo: three independent toggles plus a summary line.
//!
//! Shows the immediate-mode pattern: the host owns each `bool`, passes it
//! into `Checkbox::new`, and flips it in `update` when `on_toggle` fires.

extern crate alloc;
use alloc::string::{String, ToString};
use core::fmt::Write as _;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const WIFI_ID: WidgetId = WidgetId::new(0x610);
const BLUETOOTH_ID: WidgetId = WidgetId::new(0x611);
const AIRPLANE_ID: WidgetId = WidgetId::new(0x612);

#[derive(Clone)]
enum Msg {
    SetA(bool),
    SetB(bool),
    SetC(bool),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    a: bool,
    b: bool,
    c: bool,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            a: true,
            b: false,
            c: false,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Checkbox"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let mut summary = String::new();
        let _ = write!(
            &mut summary,
            "checked: {}",
            [
                ("Wi-Fi", self.a),
                ("Bluetooth", self.b),
                ("Airplane", self.c)
            ]
            .iter()
            .filter(|(_, on)| *on)
            .map(|(n, _)| *n)
            .collect::<alloc::vec::Vec<_>>()
            .join(", ")
        );

        Column::new()
            .spacing(10)
            .push(
                Text::new("Settings".to_string())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                Checkbox::new(self.a)
                    .id(WIFI_ID)
                    .label("Wi-Fi")
                    .on_toggle(Msg::SetA)
                    .height(Length::Fixed(24)),
            )
            .push(
                Checkbox::new(self.b)
                    .id(BLUETOOTH_ID)
                    .label("Bluetooth")
                    .on_toggle(Msg::SetB)
                    .height(Length::Fixed(24)),
            )
            .push(
                Checkbox::new(self.c)
                    .id(AIRPLANE_ID)
                    .label("Airplane mode")
                    .on_toggle(Msg::SetC)
                    .height(Length::Fixed(24)),
            )
            .push(horizontal_divider())
            .push(
                Text::new(summary)
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
            Msg::SetA(v) => s.a = v,
            Msg::SetB(v) => s.b = v,
            Msg::SetC(v) => s.c = v,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Checkbox").await;
}
