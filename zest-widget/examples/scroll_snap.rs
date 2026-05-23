//! Snap-to-center carousel: a horizontal `Column`-of-cards that settles each
//! release so a card lands centered in the viewport.
//!
//! Demonstrates [`SnapMode::Center`]: dragging pans the row of cards 1:1, and
//! on release the spring settles to the nearest snap line (a card center
//! aligned to the viewport center). The snap lines for the animation tick are
//! captured from the [`ScrollMsg::Release`] the container emits, so `update()`
//! never needs layout access.

extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Tapped(usize),
    Scroll(ScrollMsg),
    ScrollTick,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    scroll: ScrollState,
    last_tick: Instant,
    snap_lines: Vec<i32>,
    tapped: Option<usize>,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            scroll: ScrollState::new(),
            last_tick: Instant::now(),
            snap_lines: Vec::new(),
            tapped: None,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Scroll Snap"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label = match self.tapped {
            Some(i) => format!("Card {i}"),
            None => "Swipe — releases snap to center".into(),
        };
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        // Note: a horizontal carousel uses ScrollDirection::Horizontal; this
        // Column lays cards out vertically but scrolls/snaps on the chosen
        // axis. For a horizontal feel we use a Row of cards wrapped in a
        // scrollable Column-style host; here the Column hosts tall cards and
        // snaps vertically to keep the example to one container type.
        let mut carousel = Column::new()
            .spacing(8)
            .height(Length::Fill)
            .scrollable(ScrollDirection::Vertical)
            .scroll_state(&self.scroll)
            .scrollbar(ScrollbarMode::Auto)
            .snap(SnapMode::Center)
            .on_scroll(Msg::Scroll);
        for i in 0..8usize {
            let class = if Some(i) == self.tapped {
                ButtonClass::Suggested
            } else {
                ButtonClass::Standard
            };
            carousel = carousel.push(
                Button::new(format!("Card {i}"))
                    .height(Length::Fixed(120))
                    .class(class)
                    .on_press(Msg::Tapped(i)),
            );
        }

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(carousel)
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
            Msg::Tapped(i) => {
                self.screen.tapped = Some(i);
                Task::none()
            }
            Msg::Scroll(sm) => {
                // Capture snap lines from a release so the tick loop can settle
                // to a centered card without touching layout.
                if let ScrollMsg::Release { snap_lines, .. } = &sm {
                    self.screen.snap_lines = snap_lines.clone();
                }
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
                let lines = self.screen.snap_lines.clone();
                self.screen.scroll.tick(dt, SnapMode::Center, &lines);
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
    zest::run::<App>("zest - Scroll Snap").await;
}
