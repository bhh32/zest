use crate::{Renderer, Theme, renderer::RenderError};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};

/// Implemented by every widget. Object-safe (no generic methods, no associated
/// types other than implicit trait param).
pub trait Widget<'a, C: PixelColor, M: Clone> {
    /// Current bounding rectangle.
    fn rect(&self) -> Rectangle;
    /// Update the bounding rectangle. Called by layout containers.
    fn set_rect(&mut self, rect: Rectangle);
    /// Process a tap. Return Some(message) if consumed.
    fn handle_touch(&mut self, point: Point) -> Option<M>;
    /// Render through a Renderer.
    fn draw(&self, renderer: &mut dyn Renderer<C>, theme: &Theme<'a, C>)
    -> Result<(), RenderError>;
}
