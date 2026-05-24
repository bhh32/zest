//! Roller (wheel / drum selector): a vertically scrollable drum of options
//! that settles each release so the chosen option lands centered under a
//! fixed highlight band.
//!
//! Scroll state and momentum are wired exactly as in `scroll_list`: the host
//! owns a [`ScrollState`], mutates it only in `update`, and drives momentum
//! with a self-rescheduling [`tick_task`]. Because the roller snaps to center,
//! the tick uses [`SnapMode::Center`] with the roller's per-option snap lines,
//! captured from the [`ScrollMsg::Release`] the widget emits. The centered
//! option index is reported through `on_select`.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const OPTIONS: &[&str] = &[
    "January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December",
];
const ITEM_HEIGHT: u32 = 36;

#[derive(Clone)]
enum Msg {
    Selected(usize),
    Scroll(ScrollMsg),
    ScrollTick,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    scroll: ScrollState,
    last_tick: Instant,
    snap_lines: Vec<i32>,
    selected: usize,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            scroll: ScrollState::new(),
            last_tick: Instant::now(),
            snap_lines: Vec::new(),
            selected: 0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Roller"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label: String = format!("Month: {}", OPTIONS[self.selected]);
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let roller = Roller::new()
            .options(OPTIONS)
            .item_height(ITEM_HEIGHT)
            .visible_count(5)
            .selected(self.selected)
            .scroll_state(&self.scroll)
            .on_select(Msg::Selected)
            .on_scroll(Msg::Scroll);

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(roller)
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
            Msg::Selected(i) => {
                self.screen.selected = i;
                Task::none()
            }
            Msg::Scroll(sm) => {
                // Capture snap lines from the release so the tick loop can
                // settle the centered option without touching layout.
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
                // Follow the drum as it is dragged.
                self.screen.selected =
                    Roller::<Rgb565, Msg>::centered_for(&self.screen.scroll, ITEM_HEIGHT, OPTIONS.len());
                Task::none()
            }
            Msg::ScrollTick => {
                let now = Instant::now();
                let dt = (now - self.screen.last_tick).as_millis() as u32;
                self.screen.last_tick = now;
                let lines = self.screen.snap_lines.clone();
                self.screen.scroll.tick(dt, SnapMode::Center, &lines);
                self.screen.selected =
                    Roller::<Rgb565, Msg>::centered_for(&self.screen.scroll, ITEM_HEIGHT, OPTIONS.len());
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
    zest::run::<App>("zest - Roller").await;
}
