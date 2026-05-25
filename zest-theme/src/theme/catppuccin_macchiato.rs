//! Catppuccin Macchiato — official palette (catppuccin/catppuccin).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x24, 0x27, 0x3a);
const SURFACE: Rgb888 = Rgb888::new(0x1e, 0x20, 0x30);
const ELEVATED: Rgb888 = Rgb888::new(0x36, 0x3a, 0x4f);
const TEXT: Rgb888 = Rgb888::new(0xca, 0xd3, 0xf5);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xa5, 0xad, 0xcb);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x6e, 0x73, 0x8d);
const BORDER: Rgb888 = Rgb888::new(0x49, 0x4d, 0x64);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x36, 0x3a, 0x4f);
const ACCENT: Rgb888 = Rgb888::new(0x8a, 0xad, 0xf4);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x79, 0x98, 0xd7);
const SUCCESS: Rgb888 = Rgb888::new(0xa6, 0xda, 0x95);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xaf, 0xde, 0xa0);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xed, 0x87, 0x96);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xef, 0x93, 0xa0);
const WARNING: Rgb888 = Rgb888::new(0xee, 0xd4, 0x9f);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xf0, 0xd8, 0xa9);
const BLUE: Rgb888 = Rgb888::new(0x8a, 0xad, 0xf4);
const GREEN: Rgb888 = Rgb888::new(0xa6, 0xda, 0x95);
const RED: Rgb888 = Rgb888::new(0xed, 0x87, 0x96);
const YELLOW: Rgb888 = Rgb888::new(0xee, 0xd4, 0x9f);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Catppuccin Macchiato theme.
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
