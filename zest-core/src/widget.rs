//! Widget trait and [`Element`] wrapper.

use alloc::vec::Vec;

/// Heterogeneous boxed widget.
pub mod element;

pub use element::{Element, IntoElement};

use crate::{Constraints, Length, RenderError, Renderer, TouchPhase, UiAction, WidgetId};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_theme::Theme;

/// Object-safe widget contract.
pub trait Widget<C: PixelColor, M: Clone> {
    /// Report the size this widget wants within `constraints`.
    fn measure(&mut self, constraints: Constraints) -> Size;

    /// Per-axis sizing intent set via `.width(...)` / `.height(...)`.
    /// Containers consult this during layout to allocate fixed slots,
    /// query intrinsic sizes, and split residual space among
    /// `Fill` / `FillPortion` children. Default is `Length::Fill` on
    /// both axes.
    fn preferred_size(&self) -> (Length, Length) {
        (Length::Fill, Length::Fill)
    }

    /// Commit the final rectangle, recursively arranging any children.
    fn arrange(&mut self, rect: Rectangle);

    /// The current arranged rectangle.
    fn rect(&self) -> Rectangle;

    /// Dispatch a touch event. Returns a message if consumed.
    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M>;

    /// Set this widget's pressed flag if `point` hits. No message emit.
    /// Containers forward to children. Default no-op.
    fn mark_pressed(&mut self, point: Point) {
        let _ = point;
    }

    /// Stable id of this widget, if it participates in focus.
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    /// True if this widget should appear in focus traversal.
    fn is_focusable(&self) -> bool {
        false
    }

    /// Append focusable widget ids in traversal order.
    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        if self.is_focusable()
            && let Some(id) = self.widget_id()
        {
            out.push(id);
        }
    }

    /// Synchronize focused state against the runtime's currently-focused id.
    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        let _ = focused;
    }

    /// Handle a semantic action directed at this widget.
    fn handle_action(&mut self, action: UiAction) -> Option<M> {
        let _ = action;
        None
    }

    /// Route a semantic action to the widget with id `target`.
    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        if self.widget_id() == Some(target) {
            self.handle_action(action)
        } else {
            None
        }
    }

    /// Request a focus movement relative to `target` for directional actions.
    fn navigate_focus(&self, target: WidgetId, action: UiAction) -> Option<WidgetId> {
        let _ = (target, action);
        None
    }

    /// Rectangle currently occupied by the focusable target, if any.
    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        if self.widget_id() == Some(target) {
            Some(self.rect())
        } else {
            None
        }
    }

    /// Focusable widget id at `point`, if any.
    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        let _ = point;
        None
    }

    /// Paint into `renderer` at [`rect`](Self::rect).
    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError>;
}
