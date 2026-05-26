//! Theme types for the `zest` GUI framework.
//!
//! Mirrors libcosmic's grouped structure:
//!
//! - [`Container<C>`] holds colors for a region (background, foreground,
//!   divider) — used for [`Theme::background`], [`Theme::primary`],
//!   [`Theme::secondary`].
//! - [`Component<C>`] holds the four-state colors for an interactive
//!   element (base, hovered, pressed, focused) plus a foreground color
//!   for content drawn on top — used for [`Theme::accent`],
//!   [`Theme::button`], [`Theme::destructive`], etc.
//! - [`Palette<C>`] holds the named primitive colors (neutral_0 through
//!   neutral_10, accent_blue, etc.) the theme is derived from.
//! - [`Spacing`] holds the design-system spacing scale (space_xs through
//!   space_xxxl).
//! - [`CornerRadii`] holds the corner-radius scale (radius_xs through
//!   radius_xl).
//!
//! `Theme<'a, C>` is generic over a
//! [`embedded_graphics::pixelcolor::PixelColor`] and a lifetime for
//! font references (which are typically `&'static MonoFont<'static>`).

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

pub mod component;
pub mod container;
pub mod convert;
pub mod corner_radii;
pub mod font;
pub mod palette;
pub mod spacing;
pub mod style;
pub mod theme;
pub mod typography;

pub use component::Component;
pub use container::Container;
pub use convert::convert_theme;
pub use corner_radii::CornerRadii;
pub use font::{
    FONT_ZEST_MONO, FONT_ZEST_MONO_CAPTION, FONT_ZEST_MONO_DISPLAY, FONT_ZEST_MONO_HEADING,
};
pub use palette::Palette;
pub use spacing::Spacing;
pub use style::{ButtonAppearance, ButtonCatalog, ButtonClass, Status};
pub use theme::Theme;
pub use typography::Typography;
