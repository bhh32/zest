//! Menu: a vertical stack of selectable entries, each emitting a message when
//! tapped.
//!
//! Immediate-mode: the host owns which entry is currently selected and passes
//! it via [`Menu::selected`]; the selected entry is highlighted with the accent
//! color. Tapping an entry emits the bound message using the same
//! press-then-release semantics as [`Button`].

extern crate alloc;
use alloc::string::String;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone, Copy, PartialEq)]
enum Item {
    New,
    Open,
    Save,
    Settings,
    Quit,
}

#[derive(Clone)]
enum Msg {
    Pick(Item),
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    selected: usize,
    last: String,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            selected: 0,
            last: "—".into(),
        }
    }
}

const ITEMS: &[(&str, Item)] = &[
    ("New", Item::New),
    ("Open", Item::Open),
    ("Save", Item::Save),
    ("Settings", Item::Settings),
    ("Quit", Item::Quit),
];

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Menu"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let heading = Text::new(self.last.clone())
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let mut menu = Menu::new()
            .row_height(34)
            .width(Length::Fixed(180))
            .selected(self.selected);
        for (label, item) in ITEMS {
            menu = menu.entry(*label, Msg::Pick(*item));
        }

        Column::new()
            .spacing(6)
            .push(heading.into_element())
            .push(horizontal_divider())
            .push(menu)
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
        match m {
            Msg::Pick(item) => {
                if let Some(i) = ITEMS.iter().position(|(_, it)| *it == item) {
                    self.screen.selected = i;
                    self.screen.last = ITEMS[i].0.into();
                }
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
    zest::run::<App>("zest - Menu").await;
}
