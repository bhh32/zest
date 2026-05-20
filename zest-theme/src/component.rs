//! [`Component`]: per-state colors for an interactive element.
//!
//! Embedded-trimmed: no `hovered` (touch screens have no pointer-over
//! phase) and no `focused` (resistive panels don't carry kbd focus).

use embedded_graphics::pixelcolor::PixelColor;

/// Colors for an interactive UI element (button, tab, link).
///
/// The five fields cover the three [`Status`](crate::Status) values
/// (`base` for `Active`, `pressed` for `Pressed`, `disabled` for
/// `Disabled`) plus the foreground (`on_base`, used for label text)
/// and the border stroke.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Component<C: PixelColor> {
    /// Resting background color.
    pub base: C,
    /// Background while being pressed.
    pub pressed: C,
    /// Background when no `on_press` is bound.
    pub disabled: C,
    /// Color of content (text, icons) drawn on top of the background.
    pub on_base: C,
    /// Border stroke color.
    pub border: C,
}

impl<C: PixelColor> Component<C> {
    /// Construct a component.
    #[must_use]
    pub const fn new(base: C, pressed: C, disabled: C, on_base: C, border: C) -> Self {
        Self {
            base,
            pressed,
            disabled,
            on_base,
            border,
        }
    }
}
