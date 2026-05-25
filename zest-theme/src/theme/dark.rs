//! Dark theme — derived from Bryan's website palette.
//!
//! Source: `style.css` on the personal website.
//!
//! This is the **default theme** for zest applications.

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

// ---- Website palette ---------------------------------------------------
const BG: Rgb888 = Rgb888::new(0x0e, 0x0e, 0x10);
const BG_SUBTLE: Rgb888 = Rgb888::new(0x14, 0x14, 0x1a);
const BG_ELEVATED: Rgb888 = Rgb888::new(0x1c, 0x1c, 0x24);
const BORDER: Rgb888 = Rgb888::new(0x2a, 0x2a, 0x36);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x1e, 0x1e, 0x28);
const TEXT: Rgb888 = Rgb888::new(0xea, 0xe8, 0xe2);
const TEXT_MUTED: Rgb888 = Rgb888::new(0x8a, 0x88, 0x80);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x42, 0x40, 0x4a);
const ACCENT: Rgb888 = Rgb888::new(0xe0, 0x7b, 0x39);

// ---- Semantic colors (chosen for visual harmony with the accent) ------
const SUCCESS: Rgb888 = Rgb888::new(0x4c, 0xaf, 0x50); // tag-linux / tag-oss
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xff, 0x55, 0x55);
const WARNING: Rgb888 = Rgb888::new(0xff, 0xb8, 0x6c);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// The default dark theme. Static const, lives in flash.
pub const THEME: Theme<'static, Rgb888> = Theme {
    background: Container {
        base: BG,
        on_base: TEXT,
        divider: BORDER_LIGHT,
    },
    primary: Container {
        base: BG_SUBTLE,
        on_base: TEXT,
        divider: BORDER,
    },
    secondary: Container {
        base: BG_ELEVATED,
        on_base: TEXT,
        divider: BORDER,
    },
    accent: Component {
        base: ACCENT,
        pressed: Rgb888::new(0xc2, 0x6a, 0x2e),
        disabled: TEXT_FAINT,
        on_base: BG,
        border: ACCENT,
    },
    button: Component {
        base: BG_ELEVATED,
        pressed: Rgb888::new(0x35, 0x35, 0x42),
        disabled: BG_SUBTLE,
        on_base: TEXT,
        border: BORDER,
    },
    destructive: Component {
        base: DESTRUCTIVE,
        pressed: Rgb888::new(0xc8, 0x44, 0x44),
        disabled: TEXT_FAINT,
        on_base: BG,
        border: DESTRUCTIVE,
    },
    success: Component {
        base: SUCCESS,
        pressed: Rgb888::new(0x3e, 0x8e, 0x41),
        disabled: TEXT_FAINT,
        on_base: BG,
        border: SUCCESS,
    },
    warning: Component {
        base: WARNING,
        pressed: Rgb888::new(0xd9, 0x99, 0x55),
        disabled: TEXT_FAINT,
        on_base: BG,
        border: WARNING,
    },
    text_button: Component {
        base: BG,
        pressed: BG_SUBTLE,
        disabled: BG,
        on_base: ACCENT,
        border: BG, // invisible
    },
    icon_button: Component {
        base: BG,
        pressed: BG_SUBTLE,
        disabled: BG,
        on_base: TEXT_MUTED,
        border: BG,
    },
    palette: Palette {
        neutral_0: TEXT,
        neutral_2: Rgb888::new(0xcc, 0xc8, 0xbe),
        neutral_4: TEXT_MUTED,
        neutral_5: Rgb888::new(0x6a, 0x68, 0x60),
        neutral_6: TEXT_FAINT,
        neutral_8: BORDER,
        neutral_10: BG,
        accent_blue: Rgb888::new(0x82, 0xcf, 0xff), // tag-bevy
        accent_green: SUCCESS,
        accent_red: DESTRUCTIVE,
        accent_yellow: WARNING,
        black: Rgb888::new(0x00, 0x00, 0x00),
        white: Rgb888::new(0xff, 0xff, 0xff),
    },
    spacing: Spacing::default_small(),
    corner_radii: CornerRadii::default_small(),
    typography: crate::Typography::new(
        &crate::font::FONT_ZEST_MONO_DISPLAY,
        &crate::font::FONT_ZEST_MONO_HEADING,
        DEFAULT_FONT,
        &crate::font::FONT_ZEST_MONO_CAPTION,
    ),
    is_dark: true,
    is_high_contrast: false,
};
