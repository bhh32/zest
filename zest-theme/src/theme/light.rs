//! Light theme — warm-inverse of the website palette with contrast-adjusted accent.

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{
    mono_font::MonoFont,
    pixelcolor::Rgb888,
};

const BG: Rgb888 = Rgb888::new(0xf7, 0xf6, 0xf2);
const SURFACE: Rgb888 = Rgb888::new(0xef, 0xee, 0xe8);
const ELEVATED: Rgb888 = Rgb888::new(0xe6, 0xe4, 0xdc);
const TEXT: Rgb888 = Rgb888::new(0x1a, 0x1a, 0x1a);
const TEXT_MUTED: Rgb888 = Rgb888::new(0x5a, 0x58, 0x4f);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x98, 0x94, 0x88);
const BORDER: Rgb888 = Rgb888::new(0xc8, 0xc5, 0xb8);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0xdc, 0xd9, 0xcf);
const ACCENT: Rgb888 = Rgb888::new(0xc2, 0x5e, 0x1e);
const ACCENT_HOVER: Rgb888 = Rgb888::new(0xa6, 0x4d, 0x12);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x8a, 0x3f, 0x0e);
const SUCCESS: Rgb888 = Rgb888::new(0x3e, 0x8e, 0x41);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0x2e, 0x6b, 0x30);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xb8, 0x30, 0x2a);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0x9b, 0x26, 0x1f);
const WARNING: Rgb888 = Rgb888::new(0xc2, 0x68, 0x0e);
const WARNING_HOVER: Rgb888 = Rgb888::new(0x9e, 0x54, 0x09);
const BLUE: Rgb888 = Rgb888::new(0x00, 0x66, 0xb3);
const GREEN: Rgb888 = Rgb888::new(0x3e, 0x8e, 0x41);
const RED: Rgb888 = Rgb888::new(0xb8, 0x30, 0x2a);
const YELLOW: Rgb888 = Rgb888::new(0xc2, 0x68, 0x0e);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Light theme.
pub const THEME: Theme<'static, Rgb888> = Theme {
    background: Container { base: BG, on_base: TEXT, divider: BORDER_LIGHT },
    primary: Container { base: SURFACE, on_base: TEXT, divider: BORDER },
    secondary: Container { base: ELEVATED, on_base: TEXT, divider: BORDER },
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
    typography: crate::Typography::new(&crate::font::FONT_ZEST_MONO_HEADING, DEFAULT_FONT, &crate::font::FONT_ZEST_MONO_CAPTION),
    is_dark: false,
    is_high_contrast: false,
};
