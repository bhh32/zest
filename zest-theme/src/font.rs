//! Zest default bitmap mono fonts — GENERATED, DO NOT EDIT BY HAND.
//!
//! Hybrid: ASCII/Latin-1 glyphs are copied from embedded-graphics' stock
//! `iso_8859_15` bitmap fonts (identical to the originals);
//! symbols not in those fonts (`⌫`, `←`/`→`/`↑`/`↓`, `↵`, `✓`/`✗`,
//! checkboxes, geometric shapes, light box-drawing) are rasterized from
//! DejaVu Sans Mono. Four sizes (caption/body/heading/display) share one glyph
//! set of 240 code points. Regenerate with `tools/fontgen`.

use embedded_graphics::{
    geometry::Size,
    image::ImageRaw,
    mono_font::{DecorationDimensions, MonoFont, mapping::StrGlyphMapping},
    pixelcolor::BinaryColor,
};

/// Glyph set covered by the zest fonts, in atlas order.
const GLYPHS: StrGlyphMapping = StrGlyphMapping::new(
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ ¡¢£¤¥¦§¨©ª«¬­®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ‐‑‒–—‘’“”†‡•…‹›€℗™←↑→↓↵⌫─│┌┐└┘├┤┬┴┼■□▲▶▼◀○●★☆☐☑✓✗",
    31,
);

/// Zest caption mono font (6x10): bitmap text plus symbol glyphs.
pub const FONT_ZEST_MONO_CAPTION: MonoFont = MonoFont {
    image: ImageRaw::<BinaryColor>::new(include_bytes!("../fonts/zest_mono_caption.raw"), 96),
    glyph_mapping: &GLYPHS,
    character_size: Size::new(6, 10),
    character_spacing: 0,
    baseline: 7,
    underline: DecorationDimensions::new(7 + 2, 1),
    strikethrough: DecorationDimensions::new(10 / 2, 1),
};

/// Zest default body mono font (8x13): bitmap text plus symbol glyphs.
pub const FONT_ZEST_MONO: MonoFont = MonoFont {
    image: ImageRaw::<BinaryColor>::new(include_bytes!("../fonts/zest_mono.raw"), 128),
    glyph_mapping: &GLYPHS,
    character_size: Size::new(8, 13),
    character_spacing: 0,
    baseline: 10,
    underline: DecorationDimensions::new(10 + 2, 1),
    strikethrough: DecorationDimensions::new(13 / 2, 1),
};

/// Zest heading mono font (10x20): bitmap text plus symbol glyphs.
pub const FONT_ZEST_MONO_HEADING: MonoFont = MonoFont {
    image: ImageRaw::<BinaryColor>::new(include_bytes!("../fonts/zest_mono_heading.raw"), 160),
    glyph_mapping: &GLYPHS,
    character_size: Size::new(10, 20),
    character_spacing: 0,
    baseline: 15,
    underline: DecorationDimensions::new(15 + 2, 1),
    strikethrough: DecorationDimensions::new(20 / 2, 1),
};

/// Zest display mono font (32x64): bitmap text plus symbol glyphs.
pub const FONT_ZEST_MONO_DISPLAY: MonoFont = MonoFont {
    image: ImageRaw::<BinaryColor>::new(include_bytes!("../fonts/zest_mono_display.raw"), 512),
    glyph_mapping: &GLYPHS,
    character_size: Size::new(32, 64),
    character_spacing: 0,
    baseline: 48,
    underline: DecorationDimensions::new(48 + 2, 1),
    strikethrough: DecorationDimensions::new(64 / 2, 1),
};
