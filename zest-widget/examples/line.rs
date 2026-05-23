//! Line widget: a content-relative polyline (a zig-zag and a smoother
//! curve) drawn over the screen via `stroke_line`.

extern crate alloc;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
struct Noop;

struct Screen {
    theme: Theme<'static, Rgb565>,
    zigzag: [Point; 6],
    ramp: [Point; 5],
}

impl ScreenView<Rgb565, Noop> for Screen {
    fn name(&self) -> &'static str {
        "Line"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Noop> {
        // Each Line owns a Fill slot; points are content-relative so they
        // are offset by the slot's top-left at draw time.
        let zig = Line::new(&self.zigzag)
            .color(self.theme.accent.base)
            .width_px(2);
        let ramp = Line::new(&self.ramp)
            .color(self.theme.success.base)
            .width_px(1);

        Column::new()
            .spacing(6)
            .push(
                Text::new("Line / polyline")
                    .align_x(Horizontal::Center)
                    .height(Length::Fixed(20))
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(zig)
            .push(horizontal_divider())
            .push(ramp)
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
                    zigzag: [
                        Point::new(10, 60),
                        Point::new(60, 10),
                        Point::new(110, 60),
                        Point::new(160, 10),
                        Point::new(210, 60),
                        Point::new(260, 10),
                    ],
                    ramp: [
                        Point::new(10, 70),
                        Point::new(80, 50),
                        Point::new(150, 55),
                        Point::new(220, 20),
                        Point::new(290, 30),
                    ],
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
    zest::run::<App>("zest - Line").await;
}
