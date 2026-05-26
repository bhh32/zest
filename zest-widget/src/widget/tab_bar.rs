//! Compact horizontal tab bar. Single widget that wraps a `Row` of
//! tap-targets, highlighting the active tab via [`ButtonClass::Suggested`].
//! Defaults to `Length::Fixed(20)` height — sized for `FONT_8X13` plus
//! 2px border + 2px vertical padding.

use super::{Widget, button::Button, element::Element, row::Row};
use alloc::{string::String, vec::Vec};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase, UiAction, WidgetId};
use zest_theme::{ButtonClass, Theme};

/// Default tab-bar height.
pub const DEFAULT_HEIGHT: u32 = 20;

#[derive(Clone)]
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
    id: Option<WidgetId>,
    tabs: Vec<Tab<M>>,
    spacing: u32,
    inner: Option<Row<'a, C, M>>,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> TabBar<'a, C, M> {
    /// Build a tab bar from a list of tabs.
    pub fn new<I>(tabs: I) -> Self
    where
        I: IntoIterator<Item = Tab<M>>,
    {
        Self {
            id: None,
            tabs: tabs.into_iter().collect(),
            spacing: 2,
            inner: None,
            width: Length::Fill,
            height: Length::Fixed(DEFAULT_HEIGHT),
        }
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent. Defaults to [`DEFAULT_HEIGHT`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Set a stable base id so tabs can participate in focus traversal.
    #[must_use]
    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    /// Gap between tabs.
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    fn build_inner(&self) -> Row<'a, C, M> {
        let mut row = Row::new().spacing(self.spacing);
        for (index, tab) in self.tabs.iter().enumerate() {
            let class = if tab.active {
                ButtonClass::Suggested
            } else {
                ButtonClass::Standard
            };
            let mut button = Button::new(tab.label.clone())
                .on_press(tab.message.clone())
                .class(class);
            if let Some(id) = self.tab_id(index) {
                button = button.id(id);
            }
            row = row.push(button);
        }
        row
    }

    fn ensure_built(&mut self) {
        if self.inner.is_none() {
            self.inner = Some(self.build_inner());
        }
    }

    fn tab_id(&self, index: usize) -> Option<WidgetId> {
        self.id
            .map(|id| WidgetId::new(id.raw().wrapping_add(index as u64 + 1)))
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
        self.ensure_built();
        if let Some(inner) = self.inner.as_mut() {
            inner.arrange(rect);
        }
    }

    fn rect(&self) -> Rectangle {
        self.inner.as_ref().map_or(Rectangle::zero(), Widget::rect)
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.ensure_built();
        self.inner
            .as_mut()
            .and_then(|inner| Widget::<C, M>::handle_touch(inner, point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        self.ensure_built();
        if let Some(inner) = self.inner.as_mut() {
            Widget::<C, M>::mark_pressed(inner, point);
        }
    }

    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        for index in 0..self.tabs.len() {
            if let Some(id) = self.tab_id(index) {
                out.push(id);
            }
        }
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        self.ensure_built();
        if let Some(inner) = self.inner.as_mut() {
            inner.sync_focus(focused);
        }
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        self.ensure_built();
        self.inner
            .as_mut()
            .and_then(|inner| inner.route_action(target, action))
    }

    fn navigate_focus(&self, target: WidgetId, action: UiAction) -> Option<WidgetId> {
        self.inner
            .as_ref()
            .and_then(|inner| inner.navigate_focus(target, action))
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        self.inner
            .as_ref()
            .and_then(|inner| inner.focus_rect(target))
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        self.inner.as_ref().and_then(|inner| inner.focus_at(point))
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        if let Some(inner) = self.inner.as_ref() {
            Widget::<C, M>::draw(inner, renderer, theme)
        } else {
            Ok(())
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> From<TabBar<'a, C, M>> for Element<'a, C, M> {
    fn from(bar: TabBar<'a, C, M>) -> Self {
        Element::new(bar)
    }
}
