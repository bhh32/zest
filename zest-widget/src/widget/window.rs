//! Titled panel: a title bar (filled rect + title text, optional close
//! button) above a content area holding one child.
//!
//! `Window` is a *compound* widget — internally it composes a [`Column`] of a
//! title-bar [`Row`] (title [`Text`] plus an optional close [`Button`]) above
//! a content [`Container`] wrapping the supplied child.
//! The internal tree is (re)built in [`arrange`](Widget::arrange) from the
//! window's own fields, then the measure / touch / draw protocol is
//! forwarded to it, so the host only deals with `Window` itself.
//!
//! The close button (present only when [`Window::on_close`] is set) emits
//! the host message on release, exactly like a plain
//! [`Button`].

use super::{
    Widget,
    button::Button,
    column::Column,
    container::Container,
    element::{Element, IntoElement},
    row::Row,
    text::Text,
};
use alloc::string::String;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Horizontal, Length, RenderError, Renderer, TouchPhase, Vertical};
use zest_theme::Theme;

/// Height of the title bar in pixels.
const TITLE_BAR_H: u32 = 28;

/// Titled panel with a title bar and a single content child.
pub struct Window<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    title: String,
    on_close: Option<M>,
    child: Option<Element<'a, C, M>>,
    padding: u32,
    width: Length,
    height: Length,
    /// The composed internal widget tree, rebuilt each `arrange`.
    tree: Option<Element<'a, C, M>>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Window<'a, C, M> {
    /// Create a new empty window. The parent assigns position and size
    /// via `arrange`; use `.width(...)` / `.height(...)` to constrain the
    /// slot.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            title: String::new(),
            on_close: None,
            child: None,
            padding: 6,
            width: Length::Fill,
            height: Length::Fill,
            tree: None,
        }
    }

    /// Set the title bar text.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Show a close button in the title bar that emits `msg` on
    /// release. Omitting this hides the close button entirely.
    #[must_use]
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }

    /// Set the content child placed below the title bar.
    #[must_use]
    pub fn child<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.child = Some(Element::new(child));
        self
    }

    /// Inner padding around the content child.
    #[must_use]
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Build the internal Column tree from the current fields, consuming
    /// the title string, the close message, and the child element.
    fn build_tree(&mut self) -> Element<'a, C, M> {
        // Title bar: a Row with the title (filling) and an optional close
        // button pinned to the right.
        let title = Text::new(self.title.clone())
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut bar: Row<'a, C, M> = Row::new()
            .spacing(4)
            .height(Length::Fixed(TITLE_BAR_H))
            .push(title);

        if let Some(msg) = self.on_close.clone() {
            bar = bar.push(
                Button::new("x")
                    .on_press(msg)
                    .width(Length::Fixed(TITLE_BAR_H.saturating_sub(4)))
                    .height(Length::Fill),
            );
        }

        // Content area: a Container wrapping the child (if any), padded.
        let mut content: Container<'a, C, M> =
            Container::new().padding(self.padding).height(Length::Fill);
        if let Some(child) = self.child.take() {
            content = content.child(child);
        }

        Column::new()
            .spacing(0)
            .width(self.width)
            .height(self.height)
            .push(bar)
            .push(content)
            .into_element()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Window<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Window<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        if self.tree.is_none() {
            self.tree = Some(self.build_tree());
        }
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self
            .height
            .resolve(constraints.max.height, constraints.max.height);
        let size = constraints.clamp(Size::new(w, h));
        if let Some(tree) = self.tree.as_mut() {
            tree.measure(Constraints::loose(size));
        }
        size
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
        if self.tree.is_none() {
            self.tree = Some(self.build_tree());
        }
        if let Some(tree) = self.tree.as_mut() {
            tree.arrange(rect);
        }
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.tree
            .as_mut()
            .and_then(|tree| tree.handle_touch(point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(tree) = self.tree.as_mut() {
            tree.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        // Panel background + title-bar fill are drawn directly so the
        // window reads as a framed surface regardless of the child.
        renderer.fill_rect(self.rect, theme.primary.base)?;
        let bar = Rectangle::new(
            self.rect.top_left,
            Size::new(self.rect.size.width, TITLE_BAR_H.min(self.rect.size.height)),
        );
        renderer.fill_rect(bar, theme.secondary.base)?;
        renderer.stroke_rect(self.rect, theme.background.divider)?;

        if let Some(tree) = &self.tree {
            tree.draw(renderer, theme)?;
        }
        Ok(())
    }
}
