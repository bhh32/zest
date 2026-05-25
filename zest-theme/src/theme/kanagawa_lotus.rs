//! Kanagawa Lotus — rebelot/kanagawa.nvim 'lotus' palette (light).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0xf2, 0xec, 0xbc);
const SURFACE: Rgb888 = Rgb888::new(0xdc, 0xd5, 0xac);
const ELEVATED: Rgb888 = Rgb888::new(0xd5, 0xce, 0xa3);
const TEXT: Rgb888 = Rgb888::new(0x54, 0x54, 0x64);
const TEXT_MUTED: Rgb888 = Rgb888::new(0x43, 0x43, 0x6c);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x71, 0x6e, 0x61);
const BORDER: Rgb888 = Rgb888::new(0x8a, 0x89, 0x80);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0xd5, 0xce, 0xa3);
const ACCENT: Rgb888 = Rgb888::new(0x4d, 0x69, 0x9b);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x35, 0x49, 0x6d);
const SUCCESS: Rgb888 = Rgb888::new(0x6f, 0x89, 0x4e);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0x5a, 0x70, 0x3f);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xc8, 0x40, 0x53);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xa9, 0x36, 0x46);
const WARNING: Rgb888 = Rgb888::new(0xcc, 0x6d, 0x00);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xa6, 0x58, 0x00);
const BLUE: Rgb888 = Rgb888::new(0x4d, 0x69, 0x9b);
const GREEN: Rgb888 = Rgb888::new(0x6f, 0x89, 0x4e);
const RED: Rgb888 = Rgb888::new(0xc8, 0x40, 0x53);
const YELLOW: Rgb888 = Rgb888::new(0xcc, 0x6d, 0x00);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Kanagawa Lotus theme.
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
