//! Catppuccin Latte — official palette (catppuccin/catppuccin).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0xef, 0xf1, 0xf5);
const SURFACE: Rgb888 = Rgb888::new(0xe6, 0xe9, 0xef);
const ELEVATED: Rgb888 = Rgb888::new(0xcc, 0xd0, 0xda);
const TEXT: Rgb888 = Rgb888::new(0x4c, 0x4f, 0x69);
const TEXT_MUTED: Rgb888 = Rgb888::new(0x6c, 0x6f, 0x85);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x9c, 0xa0, 0xb0);
const BORDER: Rgb888 = Rgb888::new(0xbc, 0xc0, 0xcc);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0xcc, 0xd0, 0xda);
const ACCENT: Rgb888 = Rgb888::new(0x1e, 0x66, 0xf5);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x19, 0x54, 0xc9);
const SUCCESS: Rgb888 = Rgb888::new(0x40, 0xa0, 0x2b);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0x36, 0x88, 0x25);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xd2, 0x0f, 0x39);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xb2, 0x0d, 0x30);
const WARNING: Rgb888 = Rgb888::new(0xdf, 0x8e, 0x1d);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xbe, 0x79, 0x19);
const BLUE: Rgb888 = Rgb888::new(0x1e, 0x66, 0xf5);
const GREEN: Rgb888 = Rgb888::new(0x40, 0xa0, 0x2b);
const RED: Rgb888 = Rgb888::new(0xd2, 0x0f, 0x39);
const YELLOW: Rgb888 = Rgb888::new(0xdf, 0x8e, 0x1d);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Catppuccin Latte theme.
pub const THEME: Theme<'static, Rgb888> = Theme {
    background: Container {
        base: BG,
        on_base: TEXT,
        divider: BORDER_LIGHT,
    },
    primary: Container {
        base: SURFACE,
        on_base: TEXT,
        divider: BORDER,
    },
    secondary: Container {
        base: ELEVATED,
        on_base: TEXT,
        divider: BORDER,
    },
    accent: Component {
        base: ACCENT,
        pressed: ACCENT_PRESSED,
        disabled: TEXT_FAINT,
        on_base: BG,
        border: ACCENT,
    },
    button: Component {
        base: ELEVATED,
        pressed: SURFACE,
        disabled: SURFACE,
        on_base: TEXT,
        border: BORDER,
    },
    destructive: Component {
        base: DESTRUCTIVE,
        pressed: DESTRUCTIVE_HOVER,
        disabled: TEXT_FAINT,
        on_base: BG,
        border: DESTRUCTIVE,
    },
    success: Component {
        base: SUCCESS,
        pressed: SUCCESS_HOVER,
        disabled: TEXT_FAINT,
        on_base: BG,
        border: SUCCESS,
    },
    warning: Component {
        base: WARNING,
        pressed: WARNING_HOVER,
        disabled: TEXT_FAINT,
        on_base: BG,
        border: WARNING,
    },
    text_button: Component {
        base: BG,
        pressed: ELEVATED,
        disabled: BG,
        on_base: ACCENT,
        border: BG,
    },
    icon_button: Component {
        base: BG,
        pressed: ELEVATED,
        disabled: BG,
        on_base: TEXT_MUTED,
        border: BG,
    },
    palette: Palette {
        neutral_0: TEXT,
        neutral_2: TEXT_MUTED,
        neutral_4: TEXT_MUTED,
        neutral_5: TEXT_FAINT,
        neutral_6: TEXT_FAINT,
        neutral_8: BORDER,
        neutral_10: BG,
        accent_blue: BLUE,
        accent_green: GREEN,
        accent_red: RED,
        accent_yellow: YELLOW,
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
    is_dark: false,
    is_high_contrast: false,
};
