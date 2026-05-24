//! 2-D scrollable `Grid`: a large grid of cells panned on both axes by drag.
//!
//! Uses [`ScrollDirection::Both`] so a single drag pans freely in x and y. The
//! same LVGL-style scroll surface as `scroll_list`: tap-vs-drag threshold,
//! fling with friction, and edge spring-back. Scroll state is owned by the
//! host, mutated only in `update`, with momentum driven by a self-rescheduling
//! [`tick_task`].

extern crate alloc;
use alloc::format;
use alloc::string::String;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const COLS: u32 = 6;
const ROWS: u32 = 12;

#[derive(Clone)]
enum Msg {
    Picked(usize),
    Scroll(ScrollMsg),
    ScrollTick,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    scroll: ScrollState,
    last_tick: Instant,
    picked: Option<usize>,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            scroll: ScrollState::new(),
            last_tick: Instant::now(),
            picked: None,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Scroll Grid"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label: String = match self.picked {
            Some(i) => format!("Picked cell {i}"),
            None => "Drag in any direction, tap a cell".into(),
        };
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let mut grid = Grid::new(COLS, ROWS)
            .spacing(6)
            .width(Length::Fill)
            .height(Length::Fill)
            .scrollable(ScrollDirection::Both)
            .scroll_state(&self.scroll)
            .scrollbar(ScrollbarMode::Auto)
            .on_scroll(Msg::Scroll);
        for i in 0..(COLS * ROWS) as usize {
            let class = if Some(i) == self.picked {
                ButtonClass::Suggested
            } else {
                ButtonClass::Standard
            };
            grid = grid.push(
                Button::new(format!("{i}"))
                    .width(Length::Fixed(80))
                    .height(Length::Fixed(60))
                    .class(class)
                    .on_press(Msg::Picked(i)),
            );
        }

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(grid)
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
            Msg::Picked(i) => {
                self.screen.picked = Some(i);
                Task::none()
            }
            Msg::Scroll(sm) => {
                let now = Instant::now();
                let was = self.screen.scroll.is_animating();
                self.screen.scroll.apply(sm, now.as_millis());
                if self.screen.scroll.is_animating() && !was {
                    self.screen.last_tick = now;
                    return tick_task(Msg::ScrollTick);
                }
                Task::none()
            }
            Msg::ScrollTick => {
                let now = Instant::now();
                let dt = (now - self.screen.last_tick).as_millis() as u32;
                self.screen.last_tick = now;
                self.screen.scroll.tick(dt, SnapMode::None, &[]);
                if self.screen.scroll.is_animating() {
                    tick_task(Msg::ScrollTick)
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
    zest::run::<App>("zest - Scroll Grid").await;
}
