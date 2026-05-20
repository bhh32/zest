//! The [`Platform`] trait: the async hardware surface the runtime drives.
//!
//! Every platform implementation integrates with embassy: the runtime
//! awaits `next_event` and `render_with` as futures, and uses
//! `embassy_time::Timer` for frame pacing. The user's executor
//! (`embassy-executor`, `smol`, `tokio`, `pollster::block_on`) drives
//! the resulting outer future.

use crate::event::InputEvent;
use crate::renderer::{RenderError, Renderer};
use embedded_graphics::{pixelcolor::PixelColor, prelude::*};

/// The hardware surface the [`Runtime`](crate::runtime::Runtime) drives.
pub trait Platform {
    /// Pixel color type of the underlying display.
    type Color: PixelColor;
    /// Platform-level error type (covers display + flush errors).
    type Error;

    /// Await the next input event. Returning `None` signals shutdown —
    /// the runtime exits.
    #[allow(async_fn_in_trait)]
    async fn next_event(&mut self) -> Option<InputEvent>;

    /// Render a single frame.
    ///
    /// The closure receives a [`Renderer`] supplied by the platform.
    /// Hardware platforms typically wrap their `DrawTarget` with
    /// [`crate::renderer::DrawTargetRenderer`]; the desktop simulator
    /// substitutes an anti-aliasing renderer. The platform handles
    /// compositing (paged framebuffers, partial updates, double
    /// buffering, half-height blits) internally.
    #[allow(async_fn_in_trait)]
    async fn render_with<F>(&mut self, draw: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut dyn Renderer<Self::Color>) -> Result<(), RenderError>;

    /// Viewport size in pixels.
    fn viewport(&self) -> Size;
}
