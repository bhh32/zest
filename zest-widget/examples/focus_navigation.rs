//! Keyboard/encoder focus demo.
//!
//! Run in the simulator, then use Tab or the mouse wheel to move focus and
//! Enter to activate the focused control. Touch still works as usual.

extern crate alloc;

use alloc::{format, string::String};
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const NAV_ID: WidgetId = WidgetId::new(0x100);
const MENU_ID: WidgetId = WidgetId::new(0x200);
const LIST_ID: WidgetId = WidgetId::new(0x300);
const DROPDOWN_ID: WidgetId = WidgetId::new(0x400);
const APPLY_ID: WidgetId = WidgetId::new(0x500);
const RESET_ID: WidgetId = WidgetId::new(0x501);

const TABS: &[&str] = &["Home", "Inputs", "About"];
const MENU_ITEMS: &[&str] = &["Overview", "Audio", "Network", "Display"];
const LIST_ITEMS: &[&str] = &["Status", "Logs", "Metrics", "Storage"];
const THEMES: &[&str] = &["Dark", "Dracula", "Tokyo Night", "Nord"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScreenId {
    Home,
    Inputs,
    About,
}

#[derive(Clone)]
enum Msg {
    SelectScreen(ScreenId),
    SelectMenu(usize),
    SelectList(usize),
    ToggleThemeDropdown(bool),
    SelectTheme(usize),
    Apply,
    Reset,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    active: ScreenId,
    menu_selected: usize,
    list_selected: usize,
    theme_open: bool,
    theme_selected: usize,
    last_action: String,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
            active: ScreenId::Home,
            menu_selected: 0,
            list_selected: 0,
            theme_open: false,
            theme_selected: 0,
            last_action: "Tab or mouse wheel moves focus. Enter activates.".into(),
        }
    }

    fn active_index(&self) -> usize {
        match self.active {
            ScreenId::Home => 0,
            ScreenId::Inputs => 1,
            ScreenId::About => 2,
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Focus navigation"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let title = Text::new("Focus navigation")
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        let subtitle = Text::new(self.last_action.clone())
            .align_x(Horizontal::Center)
            .color(self.theme.palette.neutral_2);

        let tab_bar = TabBar::new([
            Tab::new(
                "Home",
                Msg::SelectScreen(ScreenId::Home),
                self.active == ScreenId::Home,
            ),
            Tab::new(
                "Inputs",
                Msg::SelectScreen(ScreenId::Inputs),
                self.active == ScreenId::Inputs,
            ),
            Tab::new(
                "About",
                Msg::SelectScreen(ScreenId::About),
                self.active == ScreenId::About,
            ),
        ])
        .id(NAV_ID)
        .height(Length::Fixed(24))
        .spacing(4);

        let mut menu = Menu::new()
            .id(MENU_ID)
            .row_height(32)
            .height(Length::Fixed(128))
            .selected(self.menu_selected);
        for (index, label) in MENU_ITEMS.iter().enumerate() {
            menu = menu.entry(
                *label,
                Msg::SelectMenu(index.min(MENU_ITEMS.len().saturating_sub(1))),
            );
        }

        let mut list = List::new()
            .id(LIST_ID)
            .height(Length::Fixed(176))
            .selected(self.list_selected)
            .dividers(true)
            .on_select(Msg::SelectList);
        for (index, label) in LIST_ITEMS.iter().enumerate() {
            list = list.item_with(Some(">"), *label, Some(format!("#{}", index + 1)));
        }

        let dropdown = Dropdown::new()
            .id(DROPDOWN_ID)
            .options(THEMES)
            .selected(self.theme_selected)
            .open(self.theme_open)
            .placeholder("Pick a theme")
            .width(Length::Fixed(200))
            .on_toggle(Msg::ToggleThemeDropdown)
            .on_select(Msg::SelectTheme);

        let actions = Row::new()
            .spacing(6)
            .push(
                Button::new("Apply")
                    .id(APPLY_ID)
                    .on_press(Msg::Apply)
                    .height(Length::Fixed(28)),
            )
            .push(
                Button::new("Reset")
                    .id(RESET_ID)
                    .on_press(Msg::Reset)
                    .class(ButtonClass::Destructive)
                    .height(Length::Fixed(28)),
            );

        let summary = Text::new(format!(
            "Tab: {}  Menu: {}  List: {}  Theme: {}",
            TABS[self.active_index()],
            MENU_ITEMS[self.menu_selected],
            LIST_ITEMS[self.list_selected],
            THEMES[self.theme_selected]
        ))
        .color(self.theme.background.on_base);

        Column::new()
            .spacing(6)
            .push(title.into_element())
            .push(subtitle.into_element())
            .push(horizontal_divider())
            .push(tab_bar)
            .push(dropdown)
            .push(summary.into_element())
            .push(Row::new().spacing(8).push(menu).push(list))
            .push(actions)
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

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::SelectScreen(screen) => {
                self.screen.active = screen;
                self.screen.last_action =
                    format!("Selected tab: {}", TABS[self.screen.active_index()]);
            }
            Msg::SelectMenu(index) => {
                self.screen.menu_selected = index;
                self.screen.last_action = format!("Selected menu row: {}", MENU_ITEMS[index]);
            }
            Msg::SelectList(index) => {
                self.screen.list_selected = index;
                self.screen.last_action = format!("Selected list row: {}", LIST_ITEMS[index]);
            }
            Msg::ToggleThemeDropdown(open) => {
                self.screen.theme_open = open;
                self.screen.last_action = if open {
                    "Opened theme dropdown".into()
                } else {
                    "Closed theme dropdown".into()
                };
            }
            Msg::SelectTheme(index) => {
                self.screen.theme_selected = index;
                self.screen.theme_open = false;
                self.screen.last_action = format!("Selected theme: {}", THEMES[index]);
            }
            Msg::Apply => {
                self.screen.last_action = format!(
                    "Applied {} / {} / {}",
                    TABS[self.screen.active_index()],
                    MENU_ITEMS[self.screen.menu_selected],
                    THEMES[self.screen.theme_selected]
                );
            }
            Msg::Reset => {
                self.screen.active = ScreenId::Home;
                self.screen.menu_selected = 0;
                self.screen.list_selected = 0;
                self.screen.theme_open = false;
                self.screen.theme_selected = 0;
                self.screen.last_action = "Reset demo state".into();
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
    zest::run::<App>("zest - Focus navigation").await;
}
