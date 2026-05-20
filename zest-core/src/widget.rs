//! Widget trait and [`Element`] wrapper.

/// Heterogeneous boxed widget.
pub mod element;

pub use element::{Element, IntoElement};

use crate::{Constraints, Length, RenderError, Renderer, TouchPhase};
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

    /// Paint into `renderer` at [`rect`](Self::rect).
    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError>;
}
