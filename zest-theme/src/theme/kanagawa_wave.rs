//! Kanagawa Wave — rebelot/kanagawa.nvim 'wave' palette.

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x1f, 0x1f, 0x28);
const SURFACE: Rgb888 = Rgb888::new(0x16, 0x16, 0x1d);
const ELEVATED: Rgb888 = Rgb888::new(0x2a, 0x2a, 0x37);
const TEXT: Rgb888 = Rgb888::new(0xdc, 0xd7, 0xba);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xc8, 0xc0, 0x93);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x72, 0x71, 0x69);
const BORDER: Rgb888 = Rgb888::new(0x2a, 0x2a, 0x37);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x16, 0x16, 0x1d);
const ACCENT: Rgb888 = Rgb888::new(0x7e, 0x9c, 0xd8);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x71, 0x8c, 0xc2);
const SUCCESS: Rgb888 = Rgb888::new(0x98, 0xbb, 0x6c);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xa7, 0xce, 0x77);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xc3, 0x40, 0x43);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xd7, 0x47, 0x4a);
const WARNING: Rgb888 = Rgb888::new(0xe6, 0xc3, 0x84);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xff, 0xd6, 0x91);
const BLUE: Rgb888 = Rgb888::new(0x7e, 0x9c, 0xd8);
const GREEN: Rgb888 = Rgb888::new(0x98, 0xbb, 0x6c);
const RED: Rgb888 = Rgb888::new(0xc3, 0x40, 0x43);
const YELLOW: Rgb888 = Rgb888::new(0xe6, 0xc3, 0x84);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Kanagawa Wave theme.
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
