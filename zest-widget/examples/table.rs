//! Scrollable `Table` of borrowed text cells with a header row.
//!
//! Demonstrates the [`Table`] widget: an evenly-columned grid of borrowed
//! `&str` cells, a pinned-looking header that stays put while the body
//! scrolls, zebra striping, tap-to-select a cell, and the shared LVGL-style
//! scroll surface. Scroll state/momentum are wired exactly as in
//! `scroll_list`.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

/// Header cells, borrowed by the table for the lifetime of the screen.
const HEADER: &[&str] = &["ID", "Name", "Score"];

/// Body rows, each a borrowed slice of cell strings.
const ROWS: &[&[&str]] = &[
    &["1", "Ada", "98"],
    &["2", "Babbage", "91"],
    &["3", "Curie", "95"],
    &["4", "Dijkstra", "88"],
    &["5", "Euler", "99"],
    &["6", "Fermat", "84"],
    &["7", "Gauss", "97"],
    &["8", "Hopper", "93"],
    &["9", "Ishango", "80"],
    &["10", "Jacobi", "86"],
    &["11", "Kepler", "90"],
    &["12", "Lovelace", "94"],
    &["13", "Maxwell", "89"],
    &["14", "Newton", "96"],
    &["15", "Ohm", "82"],
    &["16", "Pascal", "87"],
];

#[derive(Clone)]
enum Msg {
    CellTapped(usize, usize),
    Scroll(ScrollMsg),
    ScrollTick,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    scroll: ScrollState,
    last_tick: Instant,
    selected: Option<(usize, usize)>,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            scroll: ScrollState::new(),
            last_tick: Instant::now(),
            selected: None,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Table"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label: String = match self.selected {
            Some((r, c)) => format!("Tapped row {r}, col {c}"),
            None => "Drag to scroll, tap a cell".into(),
        };
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let mut table = Table::new()
            .height(Length::Fill)
            .header(HEADER)
            .rows(ROWS)
            .striped(true)
            .scrollable(ScrollDirection::Vertical)
            .scroll_state(&self.scroll)
            .scrollbar(ScrollbarMode::Auto)
            .on_select(Msg::CellTapped)
            .on_scroll(Msg::Scroll);
        if let Some((r, c)) = self.selected {
            table = table.selected(r, c);
        }

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(table)
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
        match m {
            Msg::CellTapped(r, c) => {
                self.screen.selected = Some((r, c));
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
    zest::run::<App>("zest - Table").await;
}
