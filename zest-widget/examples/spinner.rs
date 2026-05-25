//! Spinner widget demo: a rotating-arc loading indicator driven by a
//! self-rescheduling `Task::perform` tick loop.
//!
//! The spinner is passive — it only draws the arc at the host's current
//! `angle`. Rotation is produced here: every `Tick`, the angle advances
//! and `update` returns a fresh delayed task that fires the next `Tick`,
//! keeping the loop alive while spinning is on.

extern crate alloc;
use embassy_time::{Duration, Timer};
use zest::prelude::*;
use zest::zest_theme::theme::dark;

/// Degrees advanced per tick.
const STEP_DEG: i32 = 12;
/// Tick period (~60 fps would be 16ms; 40ms is a smooth, light spin).
const TICK_MS: u64 = 40;

#[derive(Clone)]
enum Msg {
    Tick,
    Toggle,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    angle: i32,
    spinning: bool,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            angle: 0,
            spinning: true,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Spinner"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let spinner: Spinner<'_, Rgb565, Msg> = Spinner::new(self.angle).arc_deg(100).width_px(8);

        let label = if self.spinning {
            "Loading…"
        } else {
            "Paused"
        };
        let caption = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.body)
            .color(self.theme.background.on_base)
            .height(Length::Fixed(18));

        let toggle = Button::new(if self.spinning { "Pause" } else { "Resume" })
            .on_press(Msg::Toggle)
            .height(Length::Fixed(34));

        Column::new()
            .spacing(8)
            .push(spinner)
            .push(caption)
            .push(toggle)
            .into_element()
    }
}

/// A delayed task that fires one `Tick` after `TICK_MS`. Returning this
/// from `update` on each tick is what keeps the animation rescheduling.
fn tick_task() -> Task<Msg> {
    Task::perform(async {
        Timer::after(Duration::from_millis(TICK_MS)).await;
        Some(Msg::Tick)
    })
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Msg;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Msg>) {
        // Kick off the spin loop immediately.
        (
            Self {
                screen: Screen::new(),
            },
            tick_task(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Tick => {
                if self.screen.spinning {
                    self.screen.angle = (self.screen.angle + STEP_DEG).rem_euclid(360);
                    return tick_task();
                }
                Task::none()
            }
            Msg::Toggle => {
                self.screen.spinning = !self.screen.spinning;
                if self.screen.spinning {
                    tick_task()
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Spinner").await;
}
