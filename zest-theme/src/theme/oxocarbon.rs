//! Oxocarbon — nyoom-engineering/oxocarbon.nvim palette (IBM Carbon based, dark variant).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x16, 0x16, 0x16);
const SURFACE: Rgb888 = Rgb888::new(0x13, 0x13, 0x13);
const ELEVATED: Rgb888 = Rgb888::new(0x39, 0x39, 0x39);
const TEXT: Rgb888 = Rgb888::new(0xf2, 0xf4, 0xf8);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xdd, 0xe1, 0xe6);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x52, 0x52, 0x52);
const BORDER: Rgb888 = Rgb888::new(0x39, 0x39, 0x39);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x26, 0x26, 0x26);
const ACCENT: Rgb888 = Rgb888::new(0x33, 0xb1, 0xff);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x2d, 0x9f, 0xe6);
const SUCCESS: Rgb888 = Rgb888::new(0x42, 0xbe, 0x65);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0x5b, 0xc9, 0x79);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xee, 0x53, 0x96);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xf1, 0x6c, 0xa6);
const WARNING: Rgb888 = Rgb888::new(0xff, 0xe9, 0x7b);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xff, 0xee, 0x95);
const BLUE: Rgb888 = Rgb888::new(0x33, 0xb1, 0xff);
const GREEN: Rgb888 = Rgb888::new(0x42, 0xbe, 0x65);
const RED: Rgb888 = Rgb888::new(0xee, 0x53, 0x96);
const YELLOW: Rgb888 = Rgb888::new(0xff, 0xe9, 0x7b);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Oxocarbon theme.
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
    is_dark: true,
    is_high_contrast: false,
};
