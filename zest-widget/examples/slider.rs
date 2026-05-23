//! Slider demo: a brightness slider with a live readout and a progress
//! bar mirroring its value.
//!
//! Immediate-mode: the host owns the `f32` value. The slider computes the
//! value directly from the touch x on press and drag, and emits it through
//! `on_change`; `update` stores it.

extern crate alloc;
use alloc::string::{String, ToString};
use core::fmt::Write as _;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    SetBrightness(f32),
    SetVolume(f32),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    brightness: f32,
    volume: f32,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            brightness: 0.6,
            volume: 30.0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Slider"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let mut readout = String::new();
        let _ = write!(
            &mut readout,
            "brightness {}%   volume {}",
            (self.brightness * 100.0) as i32,
            self.volume as i32
        );

        Column::new()
            .spacing(12)
            .push(
                Text::new("Sliders".to_string())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                Text::new("Brightness (0.0 .. 1.0)".to_string())
                    .font(self.theme.typography.caption)
                    .color(self.theme.background.divider),
            )
            .push(
                Slider::new(self.brightness)
                    .range(0.0, 1.0)
                    .on_change(Msg::SetBrightness)
                    .height(Length::Fixed(24)),
            )
            .push(
                Text::new("Volume (0 .. 100)".to_string())
                    .font(self.theme.typography.caption)
                    .color(self.theme.background.divider),
            )
            .push(
                Slider::new(self.volume)
                    .range(0.0, 100.0)
                    .on_change(Msg::SetVolume)
                    .height(Length::Fixed(24)),
            )
            .push(horizontal_divider())
            .push(
                ProgressBar::new(self.brightness)
                    .range(0.0, 1.0)
                    .height(Length::Fixed(10)),
            )
            .push(
                Text::new(readout)
                    .font(self.theme.typography.caption)
                    .color(self.theme.background.on_base),
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
            Msg::SetBrightness(v) => s.brightness = v,
            Msg::SetVolume(v) => s.volume = v,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Slider").await;
}
