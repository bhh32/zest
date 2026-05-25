//! Catppuccin Mocha — official palette (catppuccin/catppuccin).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x1e, 0x1e, 0x2e);
const SURFACE: Rgb888 = Rgb888::new(0x18, 0x18, 0x25);
const ELEVATED: Rgb888 = Rgb888::new(0x31, 0x32, 0x44);
const TEXT: Rgb888 = Rgb888::new(0xcd, 0xd6, 0xf4);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xa6, 0xad, 0xc8);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x6c, 0x70, 0x86);
const BORDER: Rgb888 = Rgb888::new(0x45, 0x47, 0x5a);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x31, 0x32, 0x44);
const ACCENT: Rgb888 = Rgb888::new(0x89, 0xb4, 0xfa);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x79, 0x9e, 0xdc);
const SUCCESS: Rgb888 = Rgb888::new(0xa6, 0xe3, 0xa1);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xaf, 0xe6, 0xaa);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xf3, 0x8b, 0xa8);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xf4, 0x97, 0xb1);
const WARNING: Rgb888 = Rgb888::new(0xf9, 0xe2, 0xaf);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xfa, 0xe5, 0xb7);
const BLUE: Rgb888 = Rgb888::new(0x89, 0xb4, 0xfa);
const GREEN: Rgb888 = Rgb888::new(0xa6, 0xe3, 0xa1);
const RED: Rgb888 = Rgb888::new(0xf3, 0x8b, 0xa8);
const YELLOW: Rgb888 = Rgb888::new(0xf9, 0xe2, 0xaf);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Catppuccin Mocha theme.
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
