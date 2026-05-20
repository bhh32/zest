//! Convert themes defined in [`Rgb888`] to any target [`PixelColor`].
//!
//! All preset themes are defined as `Theme<'static, Rgb888>` constants in
//! [`crate::theme`]. Users on a different color depth (typically `Rgb565`
//! for embedded touchscreens) call [`convert_theme`] to get a theme in
//! their target color.

use crate::{Component, Container, Palette, Theme};
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};

/// Convert a [`Theme<'a, Rgb888>`] to `Theme<'a, C>` for any `C: From<Rgb888>`.
#[must_use]
pub fn convert_theme<'a, C>(src: &Theme<'a, Rgb888>) -> Theme<'a, C>
where
    C: PixelColor + From<Rgb888>,
{
    Theme {
        background: convert_container(src.background),
        primary: convert_container(src.primary),
        secondary: convert_container(src.secondary),
        accent: convert_component(src.accent),
        button: convert_component(src.button),
        destructive: convert_component(src.destructive),
        success: convert_component(src.success),
        warning: convert_component(src.warning),
        text_button: convert_component(src.text_button),
        icon_button: convert_component(src.icon_button),
        palette: convert_palette(src.palette),
        spacing: src.spacing,
        corner_radii: src.corner_radii,
        typography: src.typography,
        is_dark: src.is_dark,
        is_high_contrast: src.is_high_contrast,
    }
}

fn convert_container<C: From<Rgb888>>(c: Container<Rgb888>) -> Container<C>
where
    C: PixelColor,
{
    Container {
        base: c.base.into(),
        on_base: c.on_base.into(),
        divider: c.divider.into(),
    }
}

fn convert_component<C: From<Rgb888>>(c: Component<Rgb888>) -> Component<C>
where
    C: PixelColor,
{
    Component {
        base: c.base.into(),
        pressed: c.pressed.into(),
        disabled: c.disabled.into(),
        on_base: c.on_base.into(),
        border: c.border.into(),
    }
}

fn convert_palette<C: From<Rgb888>>(p: Palette<Rgb888>) -> Palette<C>
where
    C: PixelColor,
{
    Palette {
        neutral_0: p.neutral_0.into(),
        neutral_2: p.neutral_2.into(),
        neutral_4: p.neutral_4.into(),
        neutral_5: p.neutral_5.into(),
        neutral_6: p.neutral_6.into(),
        neutral_8: p.neutral_8.into(),
        neutral_10: p.neutral_10.into(),
        accent_blue: p.accent_blue.into(),
        accent_green: p.accent_green.into(),
        accent_red: p.accent_red.into(),
        accent_yellow: p.accent_yellow.into(),
        black: p.black.into(),
        white: p.white.into(),
    }
}
