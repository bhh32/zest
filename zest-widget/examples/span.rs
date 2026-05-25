//! Span / SpanGroup: a paragraph of independently-styled inline runs.
//!
//! A [`Span`] is one styled run — text plus optional color and font overrides.
//! A [`SpanGroup`] lays runs out inline, left to right, wrapping at word
//! boundaries when a run would overflow the arranged width. This demo mixes
//! the heading font, the body font, and several accent colors in a single
//! flowing paragraph.

extern crate alloc;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

struct Screen {
    theme: Theme<'static, Rgb565>,
}

impl Screen {
    fn new() -> Self {
        Self {
            theme: convert_theme(&dark::THEME),
        }
    }
}

impl ScreenView<Rgb565, ()> for Screen {
    fn name(&self) -> &'static str {
        "Span"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, ()> {
        let p = &self.theme.palette;
        let ty = &self.theme.typography;

        let heading = Text::new("Rich text")
            .align_x(Horizontal::Center)
            .font(ty.heading)
            .color(self.theme.background.on_base);

        // A paragraph mixing fonts and colors. Plain runs inherit the group's
        // theme color and default font; styled runs override either.
        let paragraph = SpanGroup::new()
            .line_spacing(4)
            .push(Span::new("zest ").font(ty.heading).color(p.accent_blue))
            .push(Span::new("lays out "))
            .push(Span::new("inline ").color(p.accent_green))
            .push(Span::new("runs of text, each with its own "))
            .push(Span::new("color ").color(p.accent_red))
            .push(Span::new("and "))
            .push(Span::new("font").font(ty.caption).color(p.accent_yellow))
            .push(Span::new(", wrapping at word boundaries when a run would "))
            .push(Span::new("overflow ").color(p.accent_blue))
            .push(Span::new("the arranged width of the group."));

        Column::new()
            .spacing(8)
            .push(heading)
            .push(horizontal_divider())
            .push(
                Container::new()
                    .child(paragraph)
                    .padding(8)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = ();
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<()>) {
        (
            Self {
                screen: Screen::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, _m: ()) -> Task<()> {
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Span").await;
}
