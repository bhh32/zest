//! MessageBox: a modal dialog over a dimmed scrim with a centered card holding
//! a title, body text, and a row of action buttons.
//!
//! Immediate-mode: the host owns visibility. The `MessageBox` is rendered only
//! while `show` is set, layered on top of the underlying screen with a
//! [`Stack`]. Each card button carries the message emitted on tap (here both
//! flip the host's visibility flag); a tap on the scrim dismisses via
//! [`on_dismiss`](zest::prelude::MessageBox).

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Open,
    Confirm,
    Cancel,
    Dismiss,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    show: bool,
    status: String,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            show: false,
            status: "Tap the button".into(),
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "MessageBox"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let base = Column::new()
            .spacing(8)
            .push(
                Text::new(self.status.clone())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(
                Button::new("Delete item…")
                    .class(ButtonClass::Destructive)
                    .on_press(Msg::Open),
            );

        // Underlying screen always present; the modal is layered on top only
        // while `show` is set.
        let mut stack = Stack::new().push(base);
        if self.show {
            let modal = MessageBox::new()
                .title("Delete item?")
                .body("This action cannot be undone.")
                .button("Delete", Msg::Confirm)
                .button("Cancel", Msg::Cancel)
                .on_dismiss(Msg::Dismiss);
            stack = stack.push(modal);
        }

        stack.into_element()
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
        let s = &mut self.screen;
        match m {
            Msg::Open => s.show = true,
            Msg::Confirm => {
                s.show = false;
                s.status = "Deleted!".into();
            }
            Msg::Cancel => {
                s.show = false;
                s.status = "Cancelled".into();
            }
            Msg::Dismiss => {
                s.show = false;
                s.status = "Dismissed".into();
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
    zest::run::<App>("zest - MessageBox").await;
}
