//! Image widget dem: a generated RGB565 gradient blitted to the screen.

extern crate alloc;

use alloc::vec::Vec;
use zest::{prelude::*, zest_theme::theme::dark, zest_widget::Image};

const IMG_WIDTH: u32 = 220;
const IMG_HEIGHT: u32 = 160;

struct ImageScreen {
    theme: Theme<'static, Rgb565>,
    pixels: Vec<Rgb565>,
}

impl ImageScreen {
    fn new() -> Self {
        let mut pixels = Vec::with_capacity((IMG_WIDTH * IMG_HEIGHT) as usize);
        (0..IMG_HEIGHT).into_iter().for_each(|y| {
            (0..IMG_WIDTH).into_iter().for_each(|x| {
                // Diagonal gradient: red rises with x, green with y.
                let red = (x * 31 / IMG_WIDTH) as u8;
                let green = (y * 63 / IMG_HEIGHT) as u8;
                pixels.push(Rgb565::new(red, green, 12));
            })
        });

        Self {
            theme: convert_theme(&dark::THEME),
            pixels,
        }
    }
}

impl ScreenView<Rgb565, ()> for ImageScreen {
    fn name(&self) -> &'static str {
        "Image"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, ()> {
        Column::new()
            .spacing(4)
            .push(
                Text::new("Image Widget")
                    .align_x(Horizontal::Center)
                    .height(Length::Fixed(20))
                    .color(self.theme.background.on_base),
            )
            .push(Image::new(&self.pixels, Size::new(IMG_WIDTH, IMG_HEIGHT)))
            .into_element()
    }
}

struct App {
    screen: ImageScreen,
}

impl Application for App {
    type Message = ();
    type Color = Rgb565;
    type Screen = ImageScreen;

    fn init() -> (Self, Task<()>) {
        (
            Self {
                screen: ImageScreen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, _message: ()) -> Task<()> {
        Task::none()
    }

    fn view(&self) -> &ImageScreen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Image").await;
}
