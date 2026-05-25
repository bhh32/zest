//! Moonfly — bluz71/vim-moonfly-colors palette.

use crate::{Component, Container, CornerRadii, Palette, Spacing, Theme};
use embedded_graphics::{mono_font::MonoFont, pixelcolor::Rgb888};

const BG: Rgb888 = Rgb888::new(0x08, 0x08, 0x08);
const SURFACE: Rgb888 = Rgb888::new(0x1c, 0x1c, 0x1c);
const ELEVATED: Rgb888 = Rgb888::new(0x2e, 0x2e, 0x2e);
const TEXT: Rgb888 = Rgb888::new(0xc6, 0xc6, 0xc6);
const TEXT_MUTED: Rgb888 = Rgb888::new(0xb2, 0xb2, 0xb2);
const TEXT_FAINT: Rgb888 = Rgb888::new(0x80, 0x80, 0x80);
const BORDER: Rgb888 = Rgb888::new(0x44, 0x44, 0x44);
const BORDER_LIGHT: Rgb888 = Rgb888::new(0x1c, 0x1c, 0x1c);
const ACCENT: Rgb888 = Rgb888::new(0x80, 0xa0, 0xff);
const ACCENT_PRESSED: Rgb888 = Rgb888::new(0x6e, 0x90, 0xe6);
const SUCCESS: Rgb888 = Rgb888::new(0x8c, 0xc8, 0x5f);
const SUCCESS_HOVER: Rgb888 = Rgb888::new(0xa0, 0xd2, 0x79);
const DESTRUCTIVE: Rgb888 = Rgb888::new(0xff, 0x5d, 0x5d);
const DESTRUCTIVE_HOVER: Rgb888 = Rgb888::new(0xff, 0x77, 0x77);
const WARNING: Rgb888 = Rgb888::new(0xe3, 0xc7, 0x8a);
const WARNING_HOVER: Rgb888 = Rgb888::new(0xe9, 0xd1, 0x9f);
const BLUE: Rgb888 = Rgb888::new(0x80, 0xa0, 0xff);
const GREEN: Rgb888 = Rgb888::new(0x8c, 0xc8, 0x5f);
const RED: Rgb888 = Rgb888::new(0xff, 0x5d, 0x5d);
const YELLOW: Rgb888 = Rgb888::new(0xe3, 0xc7, 0x8a);

const DEFAULT_FONT: &MonoFont<'static> = &crate::font::FONT_ZEST_MONO;

/// Moonfly theme.
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
