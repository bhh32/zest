//! QR widget demo: render a scannable URL with integer-scaled modules and a
//! standard quiet zone.

extern crate alloc;

use zest::prelude::*;
use zest::zest_theme::theme::dracula;

const URL: &str = "https://bhh32.com";

struct Screen {
    theme: Theme<'static, Rgb565>,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dracula::THEME),
        }
    }
}

impl ScreenView<Rgb565, ()> for Screen {
    fn name(&self) -> &'static str {
        "QR"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, ()> {
        let qr = Qr::new(URL)
            .ecc(EccLevel::High)
            .width(Length::Fixed(160))
            .height(Length::Fixed(160));
        let status = if qr.error().is_some() {
            "QR encode failed"
        } else {
            "Scan with your phone"
        };

        Container::new()
            .padding(4)
            .child(
                Column::new()
                    .spacing(6)
                    .push(
                        Text::new("QR")
                            .align_x(Horizontal::Center)
                            .font(self.theme.typography.heading)
                            .color(self.theme.background.on_base),
                    )
                    .push(
                        Text::new(status)
                            .align_x(Horizontal::Center)
                            .font(self.theme.typography.caption)
                            .color(self.theme.palette.neutral_2),
                    )
                    .push(
                        Text::new(URL)
                            .align_x(Horizontal::Center)
                            .font(self.theme.typography.caption)
                            .color(self.theme.background.on_base),
                    )
                    .push(qr),
            )
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = ();
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<()>) {
        (
            Self {
                screen: Screen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, _message: ()) -> Task<()> {
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - QR").await;
}
