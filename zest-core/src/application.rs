//! [`Application`] trait + [`Task`] + [`Subscription`] + [`Recipe`].

pub mod box_future;
pub mod subscription;
pub mod task;

use crate::screen::ScreenView;
use embedded_graphics::pixelcolor::PixelColor;

pub use box_future::BoxFuture;
pub use subscription::{Recipe, Subscription};
pub use task::Task;

// ---- Application -------------------------------------------------------

/// An application running under the [`Runtime`](crate::runtime::Runtime).
///
/// libcosmic-shaped: the user defines an application *type*, not an
/// application *value*. The runtime calls [`Application::init`] to
/// obtain the initial state and a startup [`Task`] (used to kick off
/// async loads — read config, fetch initial data, etc.).
pub trait Application: Sized + 'static {
    /// Application message type. Must be `Clone` for runtime
    /// dispatch through `update`.
    type Message: Clone + 'static;
    /// Pixel color type used by this application's display.
    type Color: PixelColor;
    /// Concrete screen type — typically a user-defined enum
    /// dispatching to per-screen structs via match arms.
    type Screen: ScreenView<Self::Color, Self::Message>;

    /// Construct the initial application state and a startup task.
    /// Called once by the runtime before the event loop starts.
    fn init() -> (Self, Task<Self::Message>);

    /// Mutate state in response to a message.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;

    /// The currently-active screen (for drawing).
    fn view(&self) -> &Self::Screen;

    /// Long-lived message sources. Called once at startup and after
    /// every processed message — refresh is automatic. Identity comes
    /// from the [`Recipe`] inside, so changing what's returned
    /// (or returning `Subscription::none()`) cleanly replaces the
    /// subscription. Default: none.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }
}
