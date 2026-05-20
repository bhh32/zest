//! Dracula At Night — a darker Dracula fork (https://github.com/bceskavich/dracula-at-night).

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{
    mono_font::{iso_8859_15::{FONT_10X20, FONT_6X10, FONT_8X13}, MonoFont},
    pixelcolor::Rgb888,
};

const BG: Rgb888 = Rgb888::new(0x0e, 0x14, 0x19);
const SURFACE: Rgb888 = Rgb888::new(0x19, 0x1a, 0x21);
const ELEVATED: Rgb888 = Rgb888::new(0x21, 0x22, 0x2c);
const TEXT: Rgb888 = Rgb888::new(0xf8, 0xf8, 0xf2);
const TEXT_MUTED: Rgb888 = Rgb888::new(0x62, 0x72, 0xa4);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x42, 0x44, 0x50);
const BORDER: Rgb888 = Rgb888::new(0x44, 0x47, 0x5a);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x25, 0x33, 0x40);
const ACCENT: Rgb888 = Rgb888::new(0xbd, 0x93, 0xf9);
const ACCENT_HOVER: Rgb888 = Rgb888::new(0xca, 0xa9, 0xfa);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0xa4, 0x7d, 0xe2);
const SUCCESS: Rgb888 = Rgb888::new(0x50, 0xfa, 0x7b);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0x73, 0xfa, 0x97);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xff, 0x55, 0x55);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xff, 0x77, 0x77);
const WARNING: Rgb888 = Rgb888::new(0xff, 0xb8, 0x6c);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xff, 0xc8, 0x8a);
const BLUE: Rgb888 = Rgb888::new(0x8b, 0xe9, 0xfd);
const GREEN: Rgb888 = Rgb888::new(0x50, 0xfa, 0x7b);
const RED: Rgb888 = Rgb888::new(0xff, 0x55, 0x55);
const YELLOW: Rgb888 = Rgb888::new(0xf1, 0xfa, 0x8c);

const DEFAULT_FONT: &MonoFont<'static> = &FONT_8X13;

/// Dracula At Night theme.
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
    typography: crate::Typography::new(&FONT_10X20, DEFAULT_FONT, &FONT_6X10),
    is_dark: true,
    is_high_contrast: false,
};
