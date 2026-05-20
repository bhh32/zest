//! [`Palette`]: named primitive colors a theme is derived from.

use embedded_graphics::pixelcolor::PixelColor;

/// Named primitive colors a theme is derived from.
///
/// Mirrors libcosmic's neutral scale (`neutral_0` through `neutral_10`)
/// plus accent shades. Concrete themes typically construct their
/// [`Container`](crate::Container) and [`Component`](crate::Component)
/// fields by referencing palette entries (e.g. `accent.base = palette.blue`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Palette<C: PixelColor> {
    /// Lightest neutral (typically pure white on light themes, near-white
    /// on dark themes).
    pub neutral_0: C,
    /// Light neutral.
    pub neutral_2: C,
    /// Light-to-medium neutral.
    pub neutral_4: C,
    /// Medium neutral.
    pub neutral_5: C,
    /// Medium-to-dark neutral.
    pub neutral_6: C,
    /// Dark neutral.
    pub neutral_8: C,
    /// Darkest neutral.
    pub neutral_10: C,

    /// Accent color (typically blue).
    pub accent_blue: C,
    /// Success color (typically green).
    pub accent_green: C,
    /// Destructive color (typically red).
    pub accent_red: C,
    /// Warning color (typically yellow/orange).
    pub accent_yellow: C,

    /// Pure black.
    pub black: C,
    /// Pure white.
    pub white: C,
}
