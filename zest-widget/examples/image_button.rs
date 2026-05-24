//! ImageButton demo: a grid of buttons whose faces are generated RGB565
//! tiles, with labels and a press counter.

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const TILE: u32 = 40;

#[derive(Clone)]
enum Msg {
    Pressed(&'static str),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    red: Vec<Rgb565>,
    green: Vec<Rgb565>,
    blue: Vec<Rgb565>,
    last: String,
    count: u32,
}

/// Generate a `TILE` x `TILE` tile with a solid-ish hue plus a bright
/// diagonal so the press feedback is easy to read.
fn tile(base: Rgb565) -> Vec<Rgb565> {
    let mut px = Vec::with_capacity((TILE * TILE) as usize);
    for y in 0..TILE {
        for x in 0..TILE {
            if x == y || x + y == TILE - 1 {
                px.push(Rgb565::new(31, 63, 31));
            } else {
                px.push(base);
            }
        }
    }
    px
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            red: tile(Rgb565::new(28, 6, 6)),
            green: tile(Rgb565::new(6, 40, 8)),
            blue: tile(Rgb565::new(6, 12, 28)),
            last: "—".into(),
            count: 0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "ImageButton"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let size = Size::new(TILE, TILE);
        let heading = Text::new(alloc::format!("{}  (x{})", self.last, self.count))
            .align_x(Horizontal::Center)
            .height(Length::Fixed(24))
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let row = Row::new()
            .spacing(8)
            .push(
                ImageButton::new(&self.red, size)
                    .label("Red")
                    .class(ButtonClass::Destructive)
                    .on_press(Msg::Pressed("Red")),
            )
            .push(
                ImageButton::new(&self.green, size)
                    .label("Green")
                    .class(ButtonClass::Success)
                    .on_press(Msg::Pressed("Green")),
            )
            .push(
                ImageButton::new(&self.blue, size)
                    .label("Blue")
                    .class(ButtonClass::Suggested)
                    .on_press(Msg::Pressed("Blue")),
            );

        Column::new()
            .spacing(8)
            .push(heading)
            .push(horizontal_divider())
            .push(row)
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
        match m {
            Msg::Pressed(label) => {
                self.screen.last = label.to_string();
                self.screen.count += 1;
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
    zest::run::<App>("zest - ImageButton").await;
}
