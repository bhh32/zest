//! Radio button demo: a small exclusive group.
//!
//! Exclusivity is the host's job. The host owns a single `selected: Choice`
//! and renders one `RadioButton` per option with
//! `RadioButton::new(self.selected == option)`. Each button's `on_select`
//! carries its own `Choice`; the handler simply stores it, so picking one
//! deselects the rest.

extern crate alloc;
use alloc::string::ToString;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const SMALL_ID: WidgetId = WidgetId::new(0x620);
const MEDIUM_ID: WidgetId = WidgetId::new(0x621);
const LARGE_ID: WidgetId = WidgetId::new(0x622);

#[derive(Copy, Clone, PartialEq, Eq)]
enum Choice {
    Small,
    Medium,
    Large,
}

impl Choice {
    fn label(self) -> &'static str {
        match self {
            Choice::Small => "Small",
            Choice::Medium => "Medium",
            Choice::Large => "Large",
        }
    }
}

#[derive(Clone)]
struct Pick(Choice);

struct Screen {
    theme: Theme<'static, Rgb565>,
    selected: Choice,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            selected: Choice::Medium,
        }
    }

    fn option(&self, choice: Choice) -> RadioButton<Rgb565, Pick> {
        let id = match choice {
            Choice::Small => SMALL_ID,
            Choice::Medium => MEDIUM_ID,
            Choice::Large => LARGE_ID,
        };

        RadioButton::new(self.selected == choice)
            .id(id)
            .label(choice.label())
            .on_select(Pick(choice))
            .height(Length::Fixed(24))
    }
}

impl ScreenView<Rgb565, Pick> for Screen {
    fn name(&self) -> &'static str {
        "RadioButton"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Pick> {
        Column::new()
            .spacing(10)
            .push(
                Text::new("Size".to_string())
                    .align_x(Horizontal::Center)
                    .font(self.theme.typography.heading)
                    .color(self.theme.background.on_base),
            )
            .push(horizontal_divider())
            .push(self.option(Choice::Small))
            .push(self.option(Choice::Medium))
            .push(self.option(Choice::Large))
            .push(horizontal_divider())
            .push(
                Text::new(self.selected.label().to_string())
                    .font(self.theme.typography.caption)
                    .color(self.theme.palette.neutral_2),
            )
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Pick;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Pick>) {
        (
            Self {
                screen: Screen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, Pick(choice): Pick) -> Task<Pick> {
        self.screen.selected = choice;
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - RadioButton").await;
}
