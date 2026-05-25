//! Length variants in a horizontal Row: Fixed, Shrink, Fill, FillPortion.

extern crate alloc;
use alloc::string::String;
use core::fmt::Write as _;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
struct Picked(&'static str);

struct Screen {
    theme: Theme<'static, Rgb565>,
    last: &'static str,
}

impl ScreenView<Rgb565, Picked> for Screen {
    fn name(&self) -> &'static str {
        "Row"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Picked> {
        let mut header = String::new();
        let _ = write!(&mut header, "Last tapped: {}", self.last);

        let row = Row::new()
            .spacing(6)
            .push(
                Button::new("Fixed 40")
                    .on_press(Picked("Fixed 40"))
                    .class(ButtonClass::Standard)
                    .width(40),
            )
            .push(
                Button::new("Shrink")
                    .on_press(Picked("Shrink"))
                    .class(ButtonClass::Text)
                    .width(Length::Shrink),
            )
            .push(
                Button::new("Fill")
                    .on_press(Picked("Fill"))
                    .class(ButtonClass::Suggested)
                    .width(Length::Fill),
            )
            .push(
                Button::new("Portion 2")
                    .on_press(Picked("Portion 2"))
                    .class(ButtonClass::Success)
                    .width(Length::FillPortion(2)),
            );

        let legend = Text::new(
            "Fixed=40px. Shrink=label width. Fill=1 share of leftover. \
             Portion(2)=2 shares — twice as wide as Fill.",
        )
        .font(self.theme.typography.caption)
        .color(self.theme.palette.neutral_2);

        Column::new()
            .spacing(6)
            .push(
                Text::new(header)
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(row)
            .push(horizontal_divider())
            .push(legend)
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Picked;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Picked>) {
        (
            Self {
                screen: Screen {
                    theme: convert_theme(&dark::THEME),
                    last: "—",
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, Picked(l): Picked) -> Task<Picked> {
        self.screen.last = l;
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Row").await;
}
