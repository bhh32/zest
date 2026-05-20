//! [`Container`]: colors for a non-interactive region.

use embedded_graphics::pixelcolor::PixelColor;

/// Colors for a non-interactive UI region (background panel, primary
/// content area, secondary panel, etc.).
///
/// Used by [`Theme::background`](crate::Theme::background),
/// [`Theme::primary`](crate::Theme::primary), and
/// [`Theme::secondary`](crate::Theme::secondary).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Container<C: PixelColor> {
    /// Fill color of the region.
    pub base: C,
    /// Color of text and icons drawn on top of `base`.
    pub on_base: C,
    /// Color of subtle dividers within the region.
    pub divider: C,
}

impl<C: PixelColor> Container<C> {
    /// Construct a container with the given base, on-base, and divider colors.
    #[must_use]
    pub const fn new(base: C, on_base: C, divider: C) -> Self {
        Self {
            base,
            on_base,
            divider,
        }
    }
}
