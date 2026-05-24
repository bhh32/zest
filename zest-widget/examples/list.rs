//! Scrollable `List` of selectable rows.
//!
//! Demonstrates the [`List`] widget: rows with leading/trailing slots, row
//! dividers, a tap-to-select highlight, and the shared LVGL-style scroll
//! surface (drag-pan, fling, spring-back). Scroll state and momentum are
//! wired exactly as in `scroll_list` — [`ScrollState`] is owned by the
//! screen, mutated only in `update`, and momentum is driven by a
//! self-rescheduling [`tick_task`].

extern crate alloc;
use alloc::format;
use alloc::string::String;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

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
    selected: Option<usize>,
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
        "List"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label: String = match self.selected {
            Some(i) => format!("Selected row {i}"),
            None => "Drag to scroll, tap a row".into(),
        };
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let mut list = List::new()
            .height(Length::Fill)
            .dividers(true)
            .scrollable(ScrollDirection::Vertical)
            .scroll_state(&self.scroll)
            .scrollbar(ScrollbarMode::Auto)
            .on_select(Msg::Selected)
            .on_scroll(Msg::Scroll);
        if let Some(i) = self.selected {
            list = list.selected(i);
        }
        for i in 0..30usize {
            list = list.item_with(
                Some(">"),
                format!("Item number {i}"),
                Some(format!("#{i}")),
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
            Msg::Selected(i) => {
                self.screen.selected = Some(i);
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
    zest::run::<App>("zest - List").await;
}
