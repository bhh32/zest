//! Chart widget: a line series plus a bar series sharing one auto-scaled
//! plot, with axes, horizontal gridlines, and point markers.

extern crate alloc;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
struct Noop;

struct Screen {
    theme: Theme<'static, Rgb565>,
    temps: [i32; 12],
    sales: [i32; 12],
}

impl ScreenView<Rgb565, Noop> for Screen {
    fn name(&self) -> &'static str {
        "Chart"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Noop> {
        let chart = Chart::new()
            .bar_series_colored(&self.sales, self.theme.secondary.base)
            .line_series_colored(&self.temps, self.theme.accent.base)
            .axes(true)
            .gridlines(4)
            .points(true);

        Column::new()
            .spacing(6)
            .push(
                Text::new("Chart: line + bar")
                    .align_x(Horizontal::Center)
                    .height(Length::Fixed(20))
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(chart)
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Noop;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Noop>) {
        (
            Self {
                screen: Screen {
                    theme: convert_theme(&dark::THEME),
                    temps: [12, 18, 15, 22, 28, 35, 40, 38, 30, 24, 19, 14],
                    sales: [5, 9, 7, 12, 18, 25, 30, 27, 20, 14, 10, 6],
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, _m: Noop) -> Task<Noop> {
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Chart").await;
}
