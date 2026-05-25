//! Stack overlay container: three solid, distinctly-colored panels of
//! decreasing size layered on top of each other so the z-order is obvious.
//!
//! Draw order is insertion order (first = bottom), so the small red panel
//! sits on top of the green panel, which sits on top of the full-bleed blue
//! panel. Touch is routed top-first, so the OK button on the topmost panel
//! intercepts taps.
//!
//! `Divider` doubles as a solid color panel here — a sized one fills its rect.

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Dismiss,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    status: String,
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Stack"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let p = &self.theme.palette;

        // Layer 1 (bottom): full-bleed blue panel with a label at the top.
        let bottom = Stack::new()
            .push_aligned(
                Divider::new(Length::Fill, Length::Fill).color(p.accent_blue),
                Horizontal::Left,
                Vertical::Top,
            )
            .push_aligned(
                Text::new("1 - background (bottom)")
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Top)
                    .color(p.white),
                Horizontal::Center,
                Vertical::Top,
            );

        // Layer 2 (middle): a smaller green panel, centered.
        let middle = Stack::new()
            .width(Length::Fixed(250))
            .height(Length::Fixed(170))
            .push(Divider::new(Length::Fill, Length::Fill).color(p.accent_green))
            .push(
                Text::new("2 - middle panel")
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .color(p.white),
            );

        // Layer 3 (top): a small red card with a heading, the live status,
        // and an OK button — drawn last, so it is on top and touched first.
        let top_body = Column::new()
            .spacing(6)
            .push(
                Text::new("3 - top")
                    .font(self.theme.typography.heading)
                    .align_x(Horizontal::Center)
                    .color(p.white),
            )
            .push(
                Text::new(self.status.clone())
                    .align_x(Horizontal::Center)
                    .color(p.white),
            )
            .push(
                Button::new("OK")
                    .on_press(Msg::Dismiss)
                    .class(ButtonClass::Suggested),
            );
        let top = Stack::new()
            .width(Length::Fixed(170))
            .height(Length::Fixed(110))
            .push(Divider::new(Length::Fill, Length::Fill).color(p.accent_red))
            .push(top_body);

        Stack::new()
            // 1. blue background fills the whole stack.
            .push_aligned(bottom, Horizontal::Left, Vertical::Top)
            // 2. green panel centered, on top of the blue.
            .push(middle)
            // 3. red card centered, on top of everything; touched first.
            .push(top)
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
                    status: "tap OK".into(),
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Dismiss => self.screen.status = "dismissed!".into(),
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Stack").await;
}
