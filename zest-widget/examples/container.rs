//! Card layout: padded Container wrapping a vertical stack with actions.

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Cancel,
    Ok,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    status: String,
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Container"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let actions = Row::new()
            .spacing(6)
            .push(
                Button::new("Cancel")
                    .on_press(Msg::Cancel)
                    .class(ButtonClass::Destructive)
                    .width(Length::Shrink),
            )
            .push(horizontal_space())
            .push(
                Button::new("OK")
                    .on_press(Msg::Ok)
                    .class(ButtonClass::Suggested)
                    .width(Length::Shrink),
            );

        let card_body = Column::new()
            .spacing(6)
            .push(
                Text::new("Delete project?")
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                Text::new("All unsaved changes will be lost. This action cannot be undone.")
                    .font(self.theme.typography.body)
                    .color(self.theme.background.divider),
            )
            .push(vertical_space())
            .push(actions);

        let card = Container::new().padding(8).child(card_body);

        Column::new()
            .spacing(4)
            .push(
                Text::new(self.status.clone())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.caption)
                    .color(self.theme.background.on_base),
            )
            .push(card)
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
                screen: Screen {
                    theme: convert_theme(&dark::THEME),
                    status: "waiting for input...".into(),
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        self.screen.status = match m {
            Msg::Cancel => "Cancel tapped.".into(),
            Msg::Ok => "OK tapped.".into(),
        };
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Container").await;
}
