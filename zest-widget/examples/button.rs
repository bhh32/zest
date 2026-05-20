//! ButtonClass gallery. Every variant + enabled/disabled toggle.

extern crate alloc;
use alloc::string::{String, ToString};
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone)]
enum Msg {
    Pressed(&'static str),
    ToggleEnabled,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    last_pressed: String,
    enabled: bool,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            last_pressed: "—".into(),
            enabled: true,
        }
    }

    fn cell(&self, label: &'static str, class: ButtonClass, gated: bool) -> Button<'static, Rgb565, Msg> {
        let msg = if gated && !self.enabled {
            None
        } else {
            Some(Msg::Pressed(label))
        };
        Button::new(label).on_press_maybe(msg).class(class)
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Button"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let heading = Text::new(self.last_pressed.clone())
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let status_label = if self.enabled { "all enabled" } else { "half disabled" };
        let status = Text::new(status_label.to_string())
            .align_x(Horizontal::Center)
            .font(self.theme.typography.caption)
            .color(self.theme.background.divider);

        let grid = Grid::new(3, 3)
            .spacing(6)
            .push(self.cell("Standard", ButtonClass::Standard, false))
            .push(self.cell("Suggested", ButtonClass::Suggested, true))
            .push(self.cell("Destructive", ButtonClass::Destructive, true))
            .push(self.cell("Success", ButtonClass::Success, false))
            .push(self.cell("Warning", ButtonClass::Warning, true))
            .push(self.cell("Text", ButtonClass::Text, true))
            .push(self.cell("Icon", ButtonClass::Icon, false))
            .push(status)
            .push(
                Button::new(if self.enabled { "Disable half" } else { "Enable all" })
                    .on_press(Msg::ToggleEnabled)
                    .class(ButtonClass::Standard),
            );

        Column::new()
            .spacing(6)
            .push(heading)
            .push(horizontal_divider())
            .push(grid)
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
        let s = &mut self.screen;
        match m {
            Msg::Pressed(label) => s.last_pressed = label.into(),
            Msg::ToggleEnabled => s.enabled = !s.enabled,
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Button").await;
}
