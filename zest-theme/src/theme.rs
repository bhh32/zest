//! Top-level [`Theme`] type plus preset theme modules.

use crate::{
    ButtonAppearance, ButtonCatalog, ButtonClass, Component, Container, CornerRadii, Palette,
    Spacing, Status, Typography,
};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::PixelColor};

/// Top-level theme. The application owns one and passes it down to
/// widgets at draw time.
///
/// Widgets do not reach into the per-role component fields directly —
/// they go through the catalog traits ([`ButtonCatalog`], etc.) so a
/// future redesign of the palette doesn't ripple through every widget
/// implementation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Theme<'a, C: PixelColor> {
    // ---- Regions --------------------------------------------------------
    /// Background region (the outermost layer behind everything).
    pub background: Container<C>,
    /// Primary content region.
    pub primary: Container<C>,
    /// Secondary panel/sidebar region.
    pub secondary: Container<C>,

    // ---- Interactive components (consumed by catalog impls) -------------
    /// Standard button.
    pub button: Component<C>,
    /// Accent / suggested-action.
    pub accent: Component<C>,
    /// Destructive (red).
    pub destructive: Component<C>,
    /// Success (green).
    pub success: Component<C>,
    /// Warning (yellow/orange).
    pub warning: Component<C>,
    /// Text-only / ghost button.
    pub text_button: Component<C>,
    /// Icon-only button.
    pub icon_button: Component<C>,

    // ---- Primitives -----------------------------------------------------
    /// Named palette colors the rest of the theme is derived from.
    pub palette: Palette<C>,
    /// Spacing scale.
    pub spacing: Spacing,
    /// Corner-radius scale.
    pub corner_radii: CornerRadii,

    // ---- Typography -----------------------------------------------------
    /// Three-role mono font scale (heading / body / caption).
    pub typography: Typography<'a>,

    // ---- Flags ----------------------------------------------------------
    /// Whether this is a dark theme.
    pub is_dark: bool,
    /// Whether this is a high-contrast theme.
    pub is_high_contrast: bool,
}

impl<'a, C: PixelColor> Theme<'a, C> {
    /// Body font shortcut — what most widgets default to.
    #[must_use]
    pub fn default_font(&self) -> &'a MonoFont<'a> {
        self.typography.body
    }

    /// Look up the per-role component for a [`ButtonClass`].
    #[must_use]
    fn component_for(&self, class: ButtonClass) -> &Component<C> {
        match class {
            ButtonClass::Standard => &self.button,
            ButtonClass::Suggested => &self.accent,
            ButtonClass::Destructive => &self.destructive,
            ButtonClass::Success => &self.success,
            ButtonClass::Warning => &self.warning,
            ButtonClass::Text => &self.text_button,
            ButtonClass::Icon => &self.icon_button,
        }
    }
}

impl<'a, C: PixelColor> ButtonCatalog<C> for Theme<'a, C> {
    fn button(&self, class: ButtonClass, status: Status) -> ButtonAppearance<C> {
        let comp = self.component_for(class);
        let bg = match status {
            Status::Active => comp.base,
            Status::Pressed => comp.pressed,
            Status::Disabled => comp.disabled,
        };
        // Text/Icon classes paint no fill and no border by convention.
        let (background, border) = match class {
            ButtonClass::Text | ButtonClass::Icon => (None, None),
            _ => (Some(bg), Some(comp.border)),
        };
        ButtonAppearance {
            background,
            border,
            text: comp.on_base,
        }
    }
}

// ---- Preset theme submodules ------------------------------------------
//
// All preset themes are defined as `Theme<'static, Rgb888>` constants.
// Use [`crate::convert_theme`] to obtain a theme in your target color type.

pub mod catppuccin_frappe;
pub mod catppuccin_latte;
pub mod catppuccin_macchiato;
pub mod catppuccin_mocha;
pub mod custom;
pub mod dark;
pub mod dracula;
pub mod dracula_at_night;
pub mod ferra;
pub mod kanagawa_dragon;
pub mod kanagawa_lotus;
pub mod kanagawa_wave;
pub mod light;
pub mod moonfly;
pub mod nightfly;
pub mod nord;
pub mod oxocarbon;
pub mod tokyo_night;
pub mod tokyo_night_light;
pub mod tokyo_night_storm;
