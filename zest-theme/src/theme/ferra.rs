//! Ferra — casperstorm/ferra palette (warm muted dark).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x2b, 0x29, 0x2d);
const SURFACE: Rgb888 = Rgb888::new(0x38, 0x35, 0x39);
const ELEVATED: Rgb888 = Rgb888::new(0x4d, 0x42, 0x4b);
const TEXT: Rgb888 = Rgb888::new(0xd1, 0xd1, 0xe0);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xb1, 0xb6, 0x95);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x6f, 0x5d, 0x63);
const BORDER: Rgb888 = Rgb888::new(0x6f, 0x5d, 0x63);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x38, 0x35, 0x39);
const ACCENT: Rgb888 = Rgb888::new(0xf6, 0xb6, 0xc9);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0xdd, 0xa4, 0xb5);
const SUCCESS: Rgb888 = Rgb888::new(0xb1, 0xb6, 0x95);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xc1, 0xc5, 0xa9);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xe0, 0x6b, 0x75);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xe5, 0x83, 0x8b);
const WARNING: Rgb888 = Rgb888::new(0xf5, 0xd7, 0x6e);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xf7, 0xde, 0x88);
const BLUE: Rgb888 = Rgb888::new(0xf6, 0xb6, 0xc9);
const GREEN: Rgb888 = Rgb888::new(0xb1, 0xb6, 0x95);
const RED: Rgb888 = Rgb888::new(0xe0, 0x6b, 0x75);
const YELLOW: Rgb888 = Rgb888::new(0xf5, 0xd7, 0x6e);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Ferra theme.
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
