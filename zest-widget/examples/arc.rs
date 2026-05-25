//! Arc widget demo: a value gauge driven by +/- buttons.

extern crate alloc;
use alloc::format;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Add(f32),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    value: f32,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            value: 40.0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Arc"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let gauge: Arc<'_, Rgb565, Msg> = Arc::new(self.value, 0.0, 100.0).width_px(10);

        let readout = Text::new(format!("{:.0}%", self.value))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base)
            .height(Length::Fixed(28));

        let buttons = Row::new()
            .spacing(6)
            .height(Length::Fixed(36))
            .push(Button::new("-10").on_press(Msg::Add(-10.0)))
            .push(Button::new("-1").on_press(Msg::Add(-1.0)))
            .push(
                Button::new("+1")
                    .on_press(Msg::Add(1.0))
                    .class(ButtonClass::Suggested),
            )
            .push(
                Button::new("+10")
                    .on_press(Msg::Add(10.0))
                    .class(ButtonClass::Suggested),
            );

        Column::new()
            .spacing(6)
            .push(gauge)
            .push(readout)
            .push(buttons)
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
            Msg::Add(d) => {
                self.screen.value = (self.screen.value + d).clamp(0.0, 100.0);
            }
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Arc").await;
}
