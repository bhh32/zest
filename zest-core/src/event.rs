//! Runtime input and output events.

use embedded_graphics::prelude::*;

/// Touch event delivered to screens.
///
/// Re-exported so apps don't need a separate crate for the touch type.
/// Apps that already have their own `TouchEvent` (e.g. from a touch driver
/// crate) can implement `From` into this one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TouchEvent {
    /// Screen-space coordinates of the touch.
    pub point: Point,
    /// Whether this is a touch-down, ongoing, or release.
    pub phase: TouchPhase,
}

/// Phase of a touch event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TouchPhase {
    /// Finger just landed on the screen.
    Down,
    /// Finger is still on the screen (moved).
    Moved,
    /// Finger left the screen.
    Up,
}

/// Input from the platform main loop into the [`Runtime`](crate::runtime::Runtime).
///
/// Currently touchscreen-only; later versions may add `Timer` for animation
/// tick delivery.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    /// A touch event.
    Touch(TouchEvent),
}

/// What the runtime did this tick, instructing the platform main loop on
/// next steps.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TickResult {
    /// Nothing changed; the platform can sleep until the next event.
    Idle,
    /// Visual changed; call [`Runtime::draw`](crate::runtime::Runtime::draw)
    /// and flush.
    NeedsRedraw,
    /// Layout changed (and so the visual); a full layout + draw pass is
    /// required. The runtime handles this internally; the platform just
    /// needs to call `draw` and flush.
    NeedsLayout,
}

impl TickResult {
    /// True iff the runtime wants the platform to call `draw` this tick.
    #[must_use]
    pub fn wants_draw(self) -> bool {
        matches!(self, Self::NeedsRedraw | Self::NeedsLayout)
    }
}
