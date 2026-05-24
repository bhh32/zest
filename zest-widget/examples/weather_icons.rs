//! Weather-glyph alignment check, in a drag-scrollable column.
//!
//! Top: all ten conditions in narrow cells — they should share one visual
//! center. Below: each condition in its own wide (label + full-width icon)
//! row, so each glyph should stay centered in a wider-than-tall cell. The
//! whole list scrolls (drag / flick) since it overflows the screen.

extern crate alloc;
use alloc::string::ToString;
use embassy_time::Instant;
use zest::prelude::*;
use zest::zest_theme::theme::dark;
use zest::zest_widget::{WeatherCondition, WeatherIcon};

const ALL: [(&str, WeatherCondition); 10] = [
    ("Sunny", WeatherCondition::Sunny),
    ("Cloudy", WeatherCondition::Cloudy),
    ("Rain", WeatherCondition::Rain),
    ("Partly Cloudy", WeatherCondition::PartlyCloudy),
    ("Showers", WeatherCondition::Showers),
    ("Thunderstorm", WeatherCondition::Thunderstorm),
    ("Snow", WeatherCondition::Snow),
    ("Fog", WeatherCondition::Fog),
    ("Windy", WeatherCondition::Windy),
    ("Unknown", WeatherCondition::Unknown),
];

#[derive(Clone)]
enum Msg {
    Scroll(ScrollMsg),
    ScrollTick,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    scroll: ScrollState,
    last_tick: Instant,
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Weather Icons"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let cap = |s: &str| {
            Text::new(s.to_string())
                .font(self.theme.typography.caption)
                .color(self.theme.background.on_base)
                .height(Length::Fixed(16))
        };

        let mut narrow = Row::new().spacing(4).height(Length::Fixed(48));
        for (_, c) in ALL {
            narrow = narrow.push(WeatherIcon::new(c));
        }

        let mut col = Column::new()
            .spacing(6)
            .scrollable(ScrollDirection::Vertical)
            .scroll_state(&self.scroll)
            .scrollbar(ScrollbarMode::Auto)
            .on_scroll(Msg::Scroll)
            .push(cap("All ten - shared center:"))
            .push(narrow)
            .push(horizontal_divider())
            .push(cap("Each in a wide cell (drag to scroll):"));

        for (name, c) in ALL {
            let row = Row::new()
                .spacing(8)
                .height(Length::Fixed(72))
                .push(
                    Text::new(name.to_string())
                        .width(Length::Fixed(120))
                        .color(self.theme.background.on_base),
                )
                .push(WeatherIcon::new(c));
            col = col.push(row);
        }

        col.into_element()
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
            App {
                screen: Screen {
                    theme: convert_theme(&dark::THEME),
                    scroll: ScrollState::new(),
                    last_tick: Instant::now(),
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
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
    zest::run::<App>("zest - Weather Icons").await;
}
