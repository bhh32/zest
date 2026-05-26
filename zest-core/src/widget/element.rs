use super::Widget;
use crate::{Constraints, Length, RenderError, Renderer, TouchPhase, UiAction, WidgetId};
use alloc::{boxed::Box, vec::Vec};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_theme::Theme;

/// Heterogeneous boxed widget. `'a` is the lifetime of any data the
/// inner widget borrows from screen state.
pub struct Element<'a, C: PixelColor, M: Clone> {
    inner: Box<dyn Widget<C, M> + 'a>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Element<'a, C, M> {
    /// Wrap a concrete widget.
    pub fn new<W>(widget: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        Self {
            inner: Box::new(widget),
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Element<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        self.inner.measure(constraints)
    }

    fn preferred_size(&self) -> (Length, Length) {
        self.inner.preferred_size()
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.inner.arrange(rect);
    }

    fn rect(&self) -> Rectangle {
        self.inner.rect()
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.inner.handle_touch(point, phase)
    }

    fn mark_pressed(&mut self, point: Point) {
        self.inner.mark_pressed(point);
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
    }

    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        self.inner.collect_focusable(out);
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        self.inner.sync_focus(focused);
    }

    fn handle_action(&mut self, action: UiAction) -> Option<M> {
        self.inner.handle_action(action)
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        self.inner.route_action(target, action)
    }

    fn navigate_focus(&self, target: WidgetId, action: UiAction) -> Option<WidgetId> {
        self.inner.navigate_focus(target, action)
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        self.inner.focus_rect(target)
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        self.inner.focus_at(point)
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        self.inner.draw(renderer, theme)
    }
}

/// Conversion from a concrete widget into an [`Element`].
pub trait IntoElement<'a, C: PixelColor + 'a, M: Clone + 'a> {
    /// Wrap `self`.
    fn into_element(self) -> Element<'a, C, M>;
}

impl<'a, C, M, W> IntoElement<'a, C, M> for W
where
    C: PixelColor + 'a,
    M: Clone + 'a,
    W: Widget<C, M> + 'a,
{
    fn into_element(self) -> Element<'a, C, M> {
        Element::new(self)
    }
}
