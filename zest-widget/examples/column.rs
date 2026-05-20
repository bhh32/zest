//! Length variants in a vertical Column: Fixed, Shrink, Fill, FillPortion.

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
struct Noop;

struct Screen {
    theme: Theme<'static, Rgb565>,
}

impl Screen {
    fn cell(&self, label: &'static str, height: Length) -> Container<'static, Rgb565, Noop> {
        let body: Column<'static, Rgb565, Noop> = Column::new()
            .spacing(0)
            .push(horizontal_divider())
            .push(
                Text::new(String::from(label))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider());
        Container::new().height(height).child(body)
    }
}

impl ScreenView<Rgb565, Noop> for Screen {
    fn name(&self) -> &'static str {
        "Column"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Noop> {
        Column::new()
            .spacing(4)
            .push(
                Text::new("Column Length showcase")
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(self.cell("Fixed 30", Length::Fixed(30)))
            .push(self.cell("Shrink", Length::Shrink))
            .push(self.cell("Fill (1 share)", Length::Fill))
            .push(self.cell("Portion 2 (2 shares)", Length::FillPortion(2)))
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
    zest::run::<App>("zest - Column").await;
}
