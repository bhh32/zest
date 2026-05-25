//! Generates zest's default bitmap mono fonts.
//!
//! Two sources: characters that embedded-graphics' `iso_8859_15` bitmap
//! fonts already provide are copied verbatim (so text looks identical to the
//! stock bitmap fonts); the remaining symbols (arrows, `⌫`, checkboxes,
//! box-drawing, …) are rasterized from a monospace TTF (DejaVu Sans Mono)
//! to fill the gaps. The caption/body/heading sizes map to `FONT_6X10` /
//! `FONT_8X13` / `FONT_10X20`; the larger display size (hero text such as a
//! clock face) has no bitmap-font equivalent, so all of its glyphs are
//! rasterized from the TTF.

use ab_glyph_rasterizer::{point, Point as RPoint, Rasterizer};
use embedded_graphics::{
    geometry::{OriginDimensions, Size},
    mono_font::{iso_8859_15, MonoFont, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    Pixel,
};
use std::fs;
use ttf_parser::{Face, OutlineBuilder};

const DEFAULT_FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf";
const GLYPHS_PER_ROW: u32 = 16;

struct FontSpec {
    name: &'static str,
    raw_name: &'static str,
    role: &'static str,
    cell_w: u32,
    cell_h: u32,
    baseline: u32,
    character_spacing: u32,
    embedded: Option<&'static MonoFont<'static>>,
}

impl FontSpec {
    fn from_embedded(
        name: &'static str,
        raw_name: &'static str,
        role: &'static str,
        font: &'static MonoFont<'static>,
    ) -> Self {
        Self {
            name,
            raw_name,
            role,
            cell_w: font.character_size.width,
            cell_h: font.character_size.height,
            baseline: font.baseline,
            character_spacing: font.character_spacing,
            embedded: Some(font),
        }
    }

    const fn rasterized(
        name: &'static str,
        raw_name: &'static str,
        role: &'static str,
        cell_w: u32,
        cell_h: u32,
        baseline: u32,
    ) -> Self {
        Self {
            name,
            raw_name,
            role,
            cell_w,
            cell_h,
            baseline,
            character_spacing: 0,
            embedded: None,
        }
    }
}

/// (const name, raw filename, generation metadata for that size).
fn sizes() -> [FontSpec; 4] {
    [
        FontSpec::from_embedded(
            "FONT_ZEST_MONO_CAPTION",
            "zest_mono_caption.raw",
            "caption",
            &iso_8859_15::FONT_6X10,
        ),
        FontSpec::from_embedded(
            "FONT_ZEST_MONO",
            "zest_mono.raw",
            "default body",
            &iso_8859_15::FONT_8X13,
        ),
        FontSpec::from_embedded(
            "FONT_ZEST_MONO_HEADING",
            "zest_mono_heading.raw",
            "heading",
            &iso_8859_15::FONT_10X20,
        ),
        FontSpec::rasterized(
            "FONT_ZEST_MONO_DISPLAY",
            "zest_mono_display.raw",
            "display",
            32,
            64,
            48,
        ),
    ]
}

fn subset() -> Vec<u32> {
    let mut v: Vec<u32> = Vec::new();
    v.extend(0x20u32..=0x7E); // ASCII printable
    v.extend(0xA0u32..=0xFF); // Latin-1 supplement
    v.extend([
        0x2010, 0x2011, 0x2012, 0x2013, 0x2014, // hyphens / dashes
        0x2018, 0x2019, 0x201C, 0x201D, // curly quotes
        0x2022, 0x2020, 0x2021, 0x2026, // bullet, daggers, ellipsis
        0x2039, 0x203A, 0x20AC, 0x2122, 0x2117, // ‹ › € ™ ℗
    ]);
    v.extend([
        0x2190, 0x2191, 0x2192, 0x2193, 0x21B5, // ← ↑ → ↓ ↵
        0x232B, // ⌫
        0x2713, 0x2717, 0x2610, 0x2611, // ✓ ✗ ☐ ☑
        0x25C0, 0x25B6, 0x25B2, 0x25BC, // ◀ ▶ ▲ ▼
        0x25CF, 0x25CB, 0x25A0, 0x25A1, 0x2605, 0x2606, // ● ○ ■ □ ★ ☆
    ]);
    v.extend([
        0x2500, 0x2502, 0x250C, 0x2510, 0x2514, 0x2518, 0x251C, 0x2524, 0x252C, 0x2534, 0x253C,
    ]);
    v.sort();
    v.dedup();
    v
}

/// A `DrawTarget` that records `On` pixels into a `cell_w x cell_h` grid, used
/// to capture embedded-graphics' own glyph bitmaps.
struct Grid {
    w: u32,
    h: u32,
    bits: Vec<bool>,
}
impl Grid {
    fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            bits: vec![false; (w * h) as usize],
        }
    }
}
impl OriginDimensions for Grid {
    fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }
}
impl DrawTarget for Grid {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<BinaryColor>>,
    {
        for Pixel(p, c) in pixels {
            if c == BinaryColor::On
                && p.x >= 0
                && p.y >= 0
                && (p.x as u32) < self.w
                && (p.y as u32) < self.h
            {
                self.bits[p.y as usize * self.w as usize + p.x as usize] = true;
            }
        }
        Ok(())
    }
}

/// Capture an embedded-graphics glyph as a bit grid.
fn render_eg(font: &MonoFont, c: char, cell_w: u32, cell_h: u32) -> Vec<bool> {
    let mut grid = Grid::new(cell_w, cell_h);
    let style = MonoTextStyle::new(font, BinaryColor::On);
    let mut s = String::new();
    s.push(c);
    let _ = Text::with_baseline(
        &s,
        embedded_graphics::geometry::Point::new(0, 0),
        style,
        Baseline::Top,
    )
    .draw(&mut grid);
    grid.bits
}

// --- DejaVu fallback rasterization for symbols ----------------------------

struct Outline<'a> {
    r: &'a mut Rasterizer,
    scale: f32,
    baseline: f32,
    last: (f32, f32),
    start: (f32, f32),
}
impl<'a> Outline<'a> {
    fn p(&self, x: f32, y: f32) -> RPoint {
        point(x * self.scale, self.baseline - y * self.scale)
    }
}
impl<'a> OutlineBuilder for Outline<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.last = (x, y);
        self.start = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (a, b) = (self.p(self.last.0, self.last.1), self.p(x, y));
        self.r.draw_line(a, b);
        self.last = (x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (a, c, b) = (
            self.p(self.last.0, self.last.1),
            self.p(x1, y1),
            self.p(x, y),
        );
        self.r.draw_quad(a, c, b);
        self.last = (x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (a, c0, c1, b) = (
            self.p(self.last.0, self.last.1),
            self.p(x1, y1),
            self.p(x2, y2),
            self.p(x, y),
        );
        self.r.draw_cubic(a, c0, c1, b);
        self.last = (x, y);
    }
    fn close(&mut self) {
        let (a, b) = (
            self.p(self.last.0, self.last.1),
            self.p(self.start.0, self.start.1),
        );
        self.r.draw_line(a, b);
        self.last = self.start;
    }
}

fn render_ttf(face: &Face, c: char, cell_w: u32, cell_h: u32) -> Vec<bool> {
    let mut bits = vec![false; (cell_w * cell_h) as usize];
    let Some(gid) = face.glyph_index(c) else {
        return bits;
    };
    let asc = face.ascender() as f32;
    let desc = face.descender() as f32;
    let scale = cell_h as f32 / (asc - desc);
    let baseline = asc * scale;
    let mut rast = Rasterizer::new(cell_w as usize, cell_h as usize);
    let mut ob = Outline {
        r: &mut rast,
        scale,
        baseline,
        last: (0.0, 0.0),
        start: (0.0, 0.0),
    };
    let _ = face.outline_glyph(gid, &mut ob);
    rast.for_each_pixel_2d(|x, y, cov| {
        if cov > 0.5 && x < cell_w && y < cell_h {
            bits[(y * cell_w + x) as usize] = true;
        }
    });
    bits
}

fn pack(included: &[char], spec: &FontSpec, face: &Face) -> (Vec<u8>, u32, u32) {
    let cell_w = spec.cell_w;
    let cell_h = spec.cell_h;
    // A code point embedded-graphics lacks resolves to the replacement glyph.
    let repl = spec
        .embedded
        .map(|font| font.glyph_mapping.index('\u{2588}'));
    let rows = (included.len() as u32).div_ceil(GLYPHS_PER_ROW);
    let atlas_w = GLYPHS_PER_ROW * cell_w;
    let atlas_h = rows * cell_h;
    let stride = ((atlas_w + 7) / 8) as usize;
    let mut atlas = vec![0u8; stride * atlas_h as usize];
    let mut from_ttf = 0u32;

    for (i, &c) in included.iter().enumerate() {
        let bits = if let Some(font) = spec.embedded {
            let eg_has = c == '?' || Some(font.glyph_mapping.index(c)) != repl;
            if eg_has {
                render_eg(font, c, cell_w, cell_h)
            } else {
                from_ttf += 1;
                render_ttf(face, c, cell_w, cell_h)
            }
        } else {
            from_ttf += 1;
            render_ttf(face, c, cell_w, cell_h)
        };
        let (ox, oy) = (
            (i as u32 % GLYPHS_PER_ROW) * cell_w,
            (i as u32 / GLYPHS_PER_ROW) * cell_h,
        );
        for gy in 0..cell_h {
            for gx in 0..cell_w {
                if bits[(gy * cell_w + gx) as usize] {
                    let (px, py) = (ox + gx, oy + gy);
                    atlas[py as usize * stride + (px / 8) as usize] |= 0x80 >> (px % 8);
                }
            }
        }
    }
    (atlas, atlas_w, from_ttf)
}

fn main() {
    let root = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let fonts_dir = format!("{root}/zest-theme/fonts");
    let rs_out = format!("{root}/zest-theme/src/font.rs");
    let font_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_FONT_PATH.to_string());

    let data = fs::read(&font_path).expect("read font");
    let face = Face::parse(&data, 0).expect("parse font");

    let included: Vec<char> = subset().into_iter().filter_map(char::from_u32).collect();

    // Shared mapping (atlas order = subset order, same for all sizes).
    let mut map = String::new();
    let mut repl = 0usize;
    for (i, &c) in included.iter().enumerate() {
        if c == '?' {
            repl = i;
        }
        match c {
            '"' => map.push_str("\\\""),
            '\\' => map.push_str("\\\\"),
            c => map.push(c),
        }
    }

    let mut consts = String::new();
    for spec in sizes() {
        let (atlas, atlas_w, from_ttf) = pack(&included, &spec, &face);
        fs::write(format!("{fonts_dir}/{}", spec.raw_name), &atlas).expect("write raw");
        consts.push_str(&format!(
            "\n/// Zest {role} mono font ({cell_w}x{cell_h}): bitmap text plus symbol glyphs.\npub const {name}: MonoFont = MonoFont {{\n    image: ImageRaw::<BinaryColor>::new(include_bytes!(\"../fonts/{raw_name}\"), {atlas_w}),\n    glyph_mapping: &GLYPHS,\n    character_size: Size::new({cell_w}, {cell_h}),\n    character_spacing: {sp},\n    baseline: {baseline},\n    underline: DecorationDimensions::new({baseline} + 2, 1),\n    strikethrough: DecorationDimensions::new({cell_h} / 2, 1),\n}};\n",
            role = spec.role,
            cell_w = spec.cell_w,
            cell_h = spec.cell_h,
            name = spec.name,
            raw_name = spec.raw_name,
            sp = spec.character_spacing,
            baseline = spec.baseline,
        ));
        eprintln!(
            "{name}: {cell_w}x{cell_h} atlas_w={atlas_w} bytes={} ({from_ttf} symbols from TTF)",
            atlas.len(),
            name = spec.name,
            cell_w = spec.cell_w,
            cell_h = spec.cell_h,
        );
    }

    let rs = format!(
        r#"//! Zest default bitmap mono fonts — GENERATED, DO NOT EDIT BY HAND.
//!
//! Hybrid: ASCII/Latin-1 glyphs are copied from embedded-graphics' stock
//! `iso_8859_15` bitmap fonts (identical to the originals);
//! symbols not in those fonts (`⌫`, `←`/`→`/`↑`/`↓`, `↵`, `✓`/`✗`,
//! checkboxes, geometric shapes, light box-drawing) are rasterized from
//! DejaVu Sans Mono. Four sizes (caption/body/heading/display) share one glyph
//! set of {glyphs} code points. Regenerate with `tools/fontgen`.

use embedded_graphics::{{
    geometry::Size,
    image::ImageRaw,
    mono_font::{{mapping::StrGlyphMapping, DecorationDimensions, MonoFont}},
    pixelcolor::BinaryColor,
}};

/// Glyph set covered by the zest fonts, in atlas order.
const GLYPHS: StrGlyphMapping = StrGlyphMapping::new("{map}", {repl});
{consts}"#,
        glyphs = included.len(),
    );
    fs::write(&rs_out, rs).expect("write rs");
}
