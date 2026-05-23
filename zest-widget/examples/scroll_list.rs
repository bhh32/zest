//! Drag-to-scroll list: 40 button rows in a scrollable `Column`.
//!
//! Demonstrates the LVGL-style feel: a 1:1 drag pans the list, a tap picks a
//! row, and a drag that started on a row does NOT fire it (tap-vs-scroll
//! threshold). Releasing a fast drag flings with friction; over-dragging an
//! edge stretches then springs back. Momentum is driven by a self-rescheduling
//! [`tick_task`].

extern crate alloc;
use alloc::format;
use alloc::string::String;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

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
        "Scroll List"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label: String = match self.picked {
            Some(i) => format!("Picked row {i}"),
            None => "Drag to scroll, tap to pick".into(),
        };
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let mut list = Column::new()
            .spacing(4)
            .height(Length::Fill)
            .scrollable(ScrollDirection::Vertical)
            .scroll_state(&self.scroll)
            .scrollbar(ScrollbarMode::Auto)
            .on_scroll(Msg::Scroll);
        for i in 0..40usize {
            let class = if Some(i) == self.picked {
                ButtonClass::Suggested
            } else {
                ButtonClass::Standard
            };
            list = list.push(
                Button::new(format!("Row {i}"))
                    .height(Length::Fixed(44))
                    .class(class)
                    .on_press(Msg::Picked(i)),
            );
        }

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(list)
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
                // No snapping in this list.
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
    zest::run::<App>("zest - Scroll List").await;
}
