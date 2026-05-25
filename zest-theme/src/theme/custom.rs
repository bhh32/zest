//! Custom theme builder.
//!
//! For applications that want a theme that isn't one of the presets,
//! `CustomBuilder` provides a starting-from-Dark builder pattern.

use crate::{Component, Container, Palette, Theme, Typography};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

/// Builder for a custom theme. Starts from [`crate::theme::dark::THEME`] and
/// allows overriding individual fields.
#[derive(Clone)]
pub struct CustomBuilder<'a> {
    theme: Theme<'a, Rgb888>,
}

impl<'a> CustomBuilder<'a> {
    /// Start from the Dark theme.
    #[must_use]
    pub fn from_dark() -> Self {
        Self {
            theme: crate::theme::dark::THEME,
        }
    }

    /// Start from the Light theme.
    #[must_use]
    pub fn from_light() -> Self {
        Self {
            theme: crate::theme::light::THEME,
        }
    }

    /// Start from any existing theme.
    #[must_use]
    pub fn from_theme(theme: Theme<'a, Rgb888>) -> Self {
        Self { theme }
    }

    /// Override the accent component.
    #[must_use]
    pub fn accent(mut self, accent: Component<Rgb888>) -> Self {
        self.theme.accent = accent;
        self
    }

    /// Override the background region.
    #[must_use]
    pub fn background(mut self, bg: Container<Rgb888>) -> Self {
        self.theme.background = bg;
        self
    }

    /// Override the primary region.
    #[must_use]
    pub fn primary(mut self, primary: Container<Rgb888>) -> Self {
        self.theme.primary = primary;
        self
    }

    /// Override the palette.
    #[must_use]
    pub fn palette(mut self, palette: Palette<Rgb888>) -> Self {
        self.theme.palette = palette;
        self
    }

    /// Override the typography scale.
    #[must_use]
    pub fn typography(mut self, typography: Typography<'a>) -> Self {
        self.theme.typography = typography;
        self
    }

    /// Override only the display font for hero text.
    #[must_use]
    pub fn display_font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.theme.typography.display = font;
        self
    }

    /// Override only the body font (most common case). Display, heading,
    /// and caption are left as the inherited theme's choices.
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.theme.typography.body = font;
        self
    }

    /// Set the is_dark flag.
    #[must_use]
    pub fn dark(mut self, is_dark: bool) -> Self {
        self.theme.is_dark = is_dark;
        self
    }

    /// Finish, returning the built theme.
    #[must_use]
    pub fn build(self) -> Theme<'a, Rgb888> {
        self.theme
    }
}
