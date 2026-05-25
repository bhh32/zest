//! Catppuccin Frappé — official palette (catppuccin/catppuccin).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x30, 0x34, 0x46);
const SURFACE: Rgb888 = Rgb888::new(0x29, 0x2c, 0x3c);
const ELEVATED: Rgb888 = Rgb888::new(0x41, 0x45, 0x59);
const TEXT: Rgb888 = Rgb888::new(0xc6, 0xd0, 0xf5);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xa5, 0xad, 0xce);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x73, 0x79, 0x94);
const BORDER: Rgb888 = Rgb888::new(0x51, 0x57, 0x6d);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x41, 0x45, 0x59);
const ACCENT: Rgb888 = Rgb888::new(0x8c, 0xaa, 0xee);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x7b, 0x96, 0xd1);
const SUCCESS: Rgb888 = Rgb888::new(0xa6, 0xd1, 0x89);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xaf, 0xd6, 0x95);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xe7, 0x82, 0x84);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xe9, 0x8e, 0x90);
const WARNING: Rgb888 = Rgb888::new(0xe5, 0xc8, 0x90);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xe8, 0xce, 0x9b);
const BLUE: Rgb888 = Rgb888::new(0x8c, 0xaa, 0xee);
const GREEN: Rgb888 = Rgb888::new(0xa6, 0xd1, 0x89);
const RED: Rgb888 = Rgb888::new(0xe7, 0x82, 0x84);
const YELLOW: Rgb888 = Rgb888::new(0xe5, 0xc8, 0x90);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Catppuccin Frappé theme.
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
