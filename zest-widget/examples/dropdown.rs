//! Dropdown selector: a field showing the current selection that reveals its
//! option list as an overlay when open.
//!
//! Immediate-mode overlay: the host owns *both* the open flag and the selected
//! index. Tapping the field emits [`on_toggle`](zest::prelude::Dropdown) with
//! the negated flag (the host flips its stored `bool`); tapping an option emits
//! [`on_select`](zest::prelude::Dropdown) with that index (the host stores it
//! and closes the list). The widget never mutates either piece of state.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const OPTIONS: &[&str] = &[
    "Red", "Orange", "Yellow", "Green", "Blue", "Indigo", "Violet",
];

#[derive(Clone)]
enum Msg {
    Toggle(bool),
    Select(usize),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    open: bool,
    selected: usize,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            open: false,
            selected: 0,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Dropdown"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let label: String = format!("Chosen: {}", OPTIONS[self.selected]);
        let heading = Text::new(label)
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let dropdown = Dropdown::new()
            .options(OPTIONS)
            .selected(self.selected)
            .open(self.open)
            .placeholder("Pick a color…")
            .width(Length::Fixed(200))
            .on_toggle(Msg::Toggle)
            .on_select(Msg::Select);

        // Put the dropdown near the top so its overlay list has room to drop.
        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(dropdown)
            .push(Space::new(Length::Fill, Length::Fill))
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
                screen: Screen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Toggle(open) => self.screen.open = open,
            Msg::Select(i) => {
                self.screen.selected = i;
                self.screen.open = false;
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
    zest::run::<App>("zest - Dropdown").await;
}
