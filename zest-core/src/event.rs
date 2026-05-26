//! Runtime input and output events.

use embedded_graphics::prelude::*;

/// State of a non-touch button-like input.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonState {
    /// The input was pressed.
    Pressed,
    /// The input was released.
    Released,
    /// The input repeated while held.
    Repeated,
}

/// Keyboard key identifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// Printable character input.
    Char(char),
    /// Enter / return.
    Enter,
    /// Escape / back.
    Escape,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
}

/// Keyboard input delivered to screens.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// Which key changed.
    pub key: Key,
    /// Press/release/repeat state.
    pub state: ButtonState,
}

/// Rotary encoder step input.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EncoderEvent {
    /// Signed step delta. Positive and negative directions are backend-defined.
    pub delta: i32,
}

/// Semantic UI action independent of any particular hardware input.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UiAction {
    /// Activate the focused control.
    Activate,
    /// Cancel, dismiss, or go back.
    Cancel,
    /// Move focus forward.
    FocusNext,
    /// Move focus backward.
    FocusPrevious,
    /// Increase a value or move right/down in a control-specific way.
    Increment,
    /// Decrease a value or move left/up in a control-specific way.
    Decrement,
    /// Move focus or selection left.
    NavigateLeft,
    /// Move focus or selection right.
    NavigateRight,
    /// Move focus or selection up.
    NavigateUp,
    /// Move focus or selection down.
    NavigateDown,
    /// Open a surface such as a dropdown or menu.
    Open,
    /// Close a surface such as a dropdown or menu.
    Close,
}

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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    /// A touch event.
    Touch(TouchEvent),
    /// A key event.
    Key(KeyEvent),
    /// A rotary encoder event.
    Encoder(EncoderEvent),
    /// A semantic UI action.
    Action(UiAction),
}

/// What the runtime did this tick, instructing the platform main loop on
/// next steps.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TickResult {
    /// Nothing changed; the platform can sleep until the next event.
    Idle,
    /// Visual changed; redraw and flush.
    NeedsRedraw,
    /// Layout changed (and so the visual); a full layout + draw pass is
    /// required. The runtime handles this internally; the platform just
    /// needs to redraw and flush.
    NeedsLayout,
}

impl TickResult {
    /// True iff the runtime wants the platform to call `draw` this tick.
    #[must_use]
    pub fn wants_draw(self) -> bool {
        matches!(self, Self::NeedsRedraw | Self::NeedsLayout)
    }
}
