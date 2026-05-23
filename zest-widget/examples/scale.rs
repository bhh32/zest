//! Scale widget demo: a linear ruler stacked over a circular gauge,
//! with the gauge overlaid by an Arc to show a live value.

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
            value: 30.0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Scale"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let ruler: Scale<'_, Rgb565, Msg> = Scale::new(0.0, 100.0)
            .mode(ScaleMode::Linear)
            .major_ticks(5)
            .minor_per_major(4);

        // A circular gauge scale (ticks + labels) over a 270° sweep.
        let gauge_scale: Scale<'_, Rgb565, Msg> = Scale::new(0.0, 100.0)
            .mode(ScaleMode::Circular)
            .major_ticks(10)
            .minor_per_major(0)
            .start_deg(225)
            .sweep_deg(-270);

        let readout = Text::new(format!("value: {:.0}", self.value))
            .align_x(Horizontal::Center)
            .font(self.theme.typography.body)
            .color(self.theme.accent.base)
            .height(Length::Fixed(18));

        let buttons = Row::new()
            .spacing(6)
            .height(Length::Fixed(34))
            .push(Button::new("-5").on_press(Msg::Add(-5.0)))
            .push(
                Button::new("+5")
                    .on_press(Msg::Add(5.0))
                    .class(ButtonClass::Suggested),
            );

        Column::new()
            .spacing(6)
            .push(
                Container::new()
                    .height(Length::Fixed(48))
                    .child(ruler),
            )
            .push(gauge_scale)
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
        (Self { screen: Screen::new() }, Task::none())
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
    zest::run::<App>("zest - Scale").await;
}
