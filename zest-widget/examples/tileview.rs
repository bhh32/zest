//! Tileview: full-area paged tiles that settle exactly one tile per swipe.
//!
//! Each tile fills the viewport; a horizontal swipe pages left/right and the
//! shared scroll engine snaps to a tile boundary on release ([`SnapMode::Start`]
//! with one snap line per tile). Scroll state/momentum are wired exactly as in
//! `scroll_list`: the host owns a [`ScrollState`], mutates it only in `update`,
//! and drives momentum with a self-rescheduling [`tick_task`]. The active tile
//! index is reported via `on_change`, and the snap lines for the tick loop are
//! captured from the [`ScrollMsg::Release`].

extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const TILE_COUNT: usize = 4;

#[derive(Clone)]
enum Msg {
    Changed(usize),
    Scroll(ScrollMsg),
    ScrollTick,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    scroll: ScrollState,
    last_tick: Instant,
    snap_lines: Vec<i32>,
    current: usize,
    viewport: Size,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            scroll: ScrollState::new(),
            last_tick: Instant::now(),
            snap_lines: Vec::new(),
            current: 0,
            viewport: Size::zero(),
        }
    }

    /// One full-bleed colored tile with a centered label.
    fn tile(&self, i: usize, color: Rgb565) -> Stack<'static, Rgb565, Msg> {
        Stack::new()
            .push(Divider::new(Length::Fill, Length::Fill).color(color))
            .push(
                Text::new(format!("Tile {i}"))
                    .font(self.theme.typography.heading)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .color(self.theme.palette.white),
            )
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Tileview"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let p = &self.theme.palette;
        let heading = Text::new(format!("Swipe — tile {} of {TILE_COUNT}", self.current + 1))
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let tiles = Tileview::new()
            .direction(ScrollDirection::Horizontal)
            .height(Length::Fill)
            .scroll_state(&self.scroll)
            .on_change(Msg::Changed)
            .on_scroll(Msg::Scroll)
            .push(self.tile(0, p.accent_blue))
            .push(self.tile(1, p.accent_green))
            .push(self.tile(2, p.accent_red))
            .push(self.tile(3, p.accent_yellow));

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(tiles)
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
            Msg::Changed(i) => {
                self.screen.current = i;
                Task::none()
            }
            Msg::Scroll(sm) => {
                if let ScrollMsg::Release { snap_lines, viewport, .. } = &sm {
                    self.screen.snap_lines = snap_lines.clone();
                    self.screen.viewport = *viewport;
                }
                let now = Instant::now();
                let was = self.screen.scroll.is_animating();
                self.screen.scroll.apply(sm, now.as_millis());
                if self.screen.scroll.is_animating() && !was {
                    self.screen.last_tick = now;
                    return tick_task(Msg::ScrollTick);
                }
                self.screen.current = Tileview::<Rgb565, Msg>::current_for(
                    &self.screen.scroll,
                    self.screen.viewport,
                    ScrollDirection::Horizontal,
                    TILE_COUNT,
                );
                Task::none()
            }
            Msg::ScrollTick => {
                let now = Instant::now();
                let dt = (now - self.screen.last_tick).as_millis() as u32;
                self.screen.last_tick = now;
                let lines = self.screen.snap_lines.clone();
                self.screen.scroll.tick(dt, SnapMode::Start, &lines);
                self.screen.current = Tileview::<Rgb565, Msg>::current_for(
                    &self.screen.scroll,
                    self.screen.viewport,
                    ScrollDirection::Horizontal,
                    TILE_COUNT,
                );
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
    zest::run::<App>("zest - Tileview").await;
}
