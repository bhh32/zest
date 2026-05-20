//! Compact horizontal tab bar. Single widget that wraps a `Row` of
//! tap-targets, highlighting the active tab via [`ButtonClass::Suggested`].
//! Defaults to `Length::Fixed(20)` height — sized for `FONT_8X13` plus
//! 2px border + 2px vertical padding.

use super::{Widget, button::Button, element::Element, row::Row};
use alloc::string::String;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::{ButtonClass, Theme};

/// Default tab-bar height.
pub const DEFAULT_HEIGHT: u32 = 20;

/// One tab's data: label + the message to emit when tapped.
pub struct Tab<M> {
    /// Visible label.
    pub label: String,
    /// Message emitted on tap.
    pub message: M,
    /// Whether this tab is currently the active one. Active tabs use
    /// [`ButtonClass::Suggested`]; inactive use [`ButtonClass::Standard`].
    pub active: bool,
}

impl<M> Tab<M> {
    /// Construct a tab.
    pub fn new(label: impl Into<String>, message: M, active: bool) -> Self {
        Self {
            label: label.into(),
            message,
            active,
        }
    }
}

/// Horizontal tab strip. Each tab is a button that emits a user
/// message on tap; the active tab uses the suggested style.
///
/// Unlike the previous design, no theme reference is needed at
/// construction — styling flows through the catalog at draw time.
pub struct TabBar<'a, C: PixelColor, M: Clone> {
    inner: Row<'a, C, M>,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> TabBar<'a, C, M> {
    /// Build a tab bar from a list of tabs.
    pub fn new<I>(tabs: I) -> Self
    where
        I: IntoIterator<Item = Tab<M>>,
    {
        let mut row = Row::new().spacing(2);
        for tab in tabs {
            let class = if tab.active {
                ButtonClass::Suggested
            } else {
                ButtonClass::Standard
            };
            row = row.push(Button::new(tab.label).on_press(tab.message).class(class));
        }
        Self {
            inner: row,
            width: Length::Fill,
            height: Length::Fixed(DEFAULT_HEIGHT),
        }
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: height sizing intent. Defaults to [`DEFAULT_HEIGHT`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Builder: gap between tabs.
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.inner = self.inner.spacing(spacing);
        self
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for TabBar<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self.height.resolve(DEFAULT_HEIGHT, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.inner.arrange(rect);
    }

    fn rect(&self) -> Rectangle {
        self.inner.rect()
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        Widget::<C, M>::handle_touch(&mut self.inner, point, phase)
    }

    fn mark_pressed(&mut self, point: Point) {
        Widget::<C, M>::mark_pressed(&mut self.inner, point);
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        Widget::<C, M>::draw(&self.inner, renderer, theme)
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> From<TabBar<'a, C, M>> for Element<'a, C, M> {
    fn from(bar: TabBar<'a, C, M>) -> Self {
        Element::new(bar)
    }
}
