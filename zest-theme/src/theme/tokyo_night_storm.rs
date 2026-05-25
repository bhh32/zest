//! Tokyo Night Storm — folke/tokyonight.nvim 'storm' variant (verified against storm.lua).
//!
//! Storm shares the 'night' accent palette (blue #7aa2f7, green #9ece6a,
//! red #f7768e, yellow #e0af68) but uses a slightly lighter background trio:
//! bg #24283b, bg_dark #1f2335, bg_highlight #292e42.

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x24, 0x28, 0x3b);
const SURFACE: Rgb888 = Rgb888::new(0x1f, 0x23, 0x35);
const ELEVATED: Rgb888 = Rgb888::new(0x29, 0x2e, 0x42);
const TEXT: Rgb888 = Rgb888::new(0xc0, 0xca, 0xf5);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xa9, 0xb1, 0xd6);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x56, 0x5f, 0x89);
const BORDER: Rgb888 = Rgb888::new(0x3b, 0x42, 0x61);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x1f, 0x23, 0x35);
const ACCENT: Rgb888 = Rgb888::new(0x7a, 0xa2, 0xf7);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x5d, 0x85, 0xd6);
const SUCCESS: Rgb888 = Rgb888::new(0x9e, 0xce, 0x6a);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xb0, 0xd9, 0x7c);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xf7, 0x76, 0x8e);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xff, 0x8a, 0xa0);
const WARNING: Rgb888 = Rgb888::new(0xe0, 0xaf, 0x68);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xeb, 0xc2, 0x83);
const BLUE: Rgb888 = Rgb888::new(0x7a, 0xa2, 0xf7);
const GREEN: Rgb888 = Rgb888::new(0x9e, 0xce, 0x6a);
const RED: Rgb888 = Rgb888::new(0xf7, 0x76, 0x8e);
const YELLOW: Rgb888 = Rgb888::new(0xe0, 0xaf, 0x68);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Tokyo Night Storm theme.
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
