//! Nord — from <https://www.nordtheme.com> (verified against nord0-nord15 palette).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x2e, 0x34, 0x40);
const SURFACE: Rgb888 = Rgb888::new(0x3b, 0x42, 0x52);
const ELEVATED: Rgb888 = Rgb888::new(0x43, 0x4c, 0x5e);
const TEXT: Rgb888 = Rgb888::new(0xec, 0xef, 0xf4);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xd8, 0xde, 0xe9);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x4c, 0x56, 0x6a);
const BORDER: Rgb888 = Rgb888::new(0x4c, 0x56, 0x6a);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x43, 0x4c, 0x5e);
const ACCENT: Rgb888 = Rgb888::new(0x88, 0xc0, 0xd0);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x5e, 0x81, 0xac);
const SUCCESS: Rgb888 = Rgb888::new(0xa3, 0xbe, 0x8c);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xb5, 0xcc, 0x9d);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xbf, 0x61, 0x6a);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xcd, 0x77, 0x83);
const WARNING: Rgb888 = Rgb888::new(0xeb, 0xcb, 0x8b);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xf0, 0xd5, 0xa0);
const BLUE: Rgb888 = Rgb888::new(0x5e, 0x81, 0xac);
const GREEN: Rgb888 = Rgb888::new(0xa3, 0xbe, 0x8c);
const RED: Rgb888 = Rgb888::new(0xbf, 0x61, 0x6a);
const YELLOW: Rgb888 = Rgb888::new(0xeb, 0xcb, 0x8b);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Nord theme.
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
