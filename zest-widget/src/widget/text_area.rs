//! Editable multi-line text field.
//!
//! `TextArea` is the display half of an editable text input. Following the
//! "zest way", it is **transient and stateless**: the host owns the text
//! (a `String`) and the cursor position (a char index), rebuilds the widget
//! each frame via [`TextArea::new`] + [`cursor`](TextArea::cursor), and
//! mutates its own state in `update`. The widget itself only *renders* and
//! *maps taps to char indices*.
//!
//! It pairs naturally with [`Keyboard`](super::keyboard::Keyboard): wire the
//! keyboard's [`KeyAction`](super::keyboard::KeyAction) messages into the
//! host's `update` to insert/delete characters and move the cursor, and wire
//! [`on_tap`](TextArea::on_tap) to reposition the cursor on touch. See
//! `examples/text_area.rs` for a complete editor.
//!
//! # Layout & wrapping
//!
//! Text is laid out into visual lines by **char-wrapping** to the available
//! pixel width using the mono font's `character_size.width`, and is also
//! broken on `'\n'`. The widget **assumes ASCII**: wrapping and the
//! tap-to-index mapping count `char`s and multiply by the fixed glyph width,
//! which is only correct for single-cell ASCII text. Non-ASCII / wide / 0-cell
//! characters will misalign the cursor and hit-testing.
//!
//! # Scrolling
//!
//! Rendering is clipped to the arranged rect via
//! [`push_clip`](Renderer::push_clip) / [`pop_clip`](Renderer::pop_clip).
//! As you type past the last visible row the view scrolls down to keep the
//! cursor's line within the bottom of the viewport. It does not scroll back
//! up when the cursor moves above the first visible line.
//!
//! # Colors
//!
//! Text defaults to `theme.background.on_base`, the cursor bar to
//! `theme.accent.base`, and the (dimmed) placeholder to
//! `theme.background.divider`. All three are overridable via builders.

use super::Widget;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Cursor bar thickness in pixels.
const CURSOR_W: u32 = 2;
/// Default intrinsic width when sized `Shrink`.
const INTRINSIC_W: u32 = 200;
/// Default intrinsic height when sized `Shrink` (≈ a few lines).
const INTRINSIC_H: u32 = 96;

/// A single visual line: a byte range into the source text plus whether it
/// ends at a hard `'\n'` (so a trailing-newline empty line can be modeled).
#[derive(Copy, Clone)]
struct VisualLine {
    /// Byte offset of the line's first char in the source text.
    start: usize,
    /// Byte offset one past the line's last *rendered* char.
    end: usize,
    /// Char index of the first char on this line.
    char_start: usize,
    /// Number of chars rendered on this line (excludes a hard newline).
    char_len: usize,
}

/// Editable multi-line text field.
///
/// Borrows the text as `Cow<'a, str>` and reads a host-owned cursor position
/// (char index). It emits a message via [`on_tap`](TextArea::on_tap) carrying
/// the nearest char index when tapped, so the host can move its cursor.
pub struct TextArea<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    text: Cow<'a, str>,
    cursor: usize,
    placeholder: Cow<'a, str>,
    color: Option<C>,
    cursor_color: Option<C>,
    placeholder_color: Option<C>,
    font: Option<&'a MonoFont<'a>>,
    on_tap: Option<Box<dyn Fn(usize) -> M + 'a>>,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> TextArea<'a, C, M> {
    /// New text area displaying `text`. The cursor defaults to index `0`;
    /// set it with [`cursor`](Self::cursor). Position and size are assigned
    /// by the parent container via `arrange`.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            rect: Rectangle::zero(),
            text: text.into(),
            cursor: 0,
            placeholder: Cow::Borrowed(""),
            color: None,
            cursor_color: None,
            placeholder_color: None,
            font: None,
            on_tap: None,
            width: Length::Fill,
            height: Length::Fill,
            _color: PhantomData,
        }
    }

    /// Cursor position as a **char index** into the text. Values
    /// past the end are clamped to the char count at draw time.
    #[must_use]
    pub fn cursor(mut self, index: usize) -> Self {
        self.cursor = index;
        self
    }

    /// Dimmed placeholder shown when the text is empty.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Override the text color (default: `theme.background.on_base`).
    #[must_use]
    pub fn color(mut self, color: C) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the cursor bar color (default: `theme.accent.base`).
    #[must_use]
    pub fn cursor_color(mut self, color: C) -> Self {
        self.cursor_color = Some(color);
        self
    }

    /// Override the placeholder color (default:
    /// `theme.background.divider`).
    #[must_use]
    pub fn placeholder_color(mut self, color: C) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    /// Override the font (default: `theme.default_font()`). Must be a
    /// mono font; wrapping relies on `character_size.width`.
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }

    /// Callback receiving the nearest char index when the area is
    /// tapped. Without it, taps are ignored. The host typically stores the
    /// returned index as its new cursor position.
    ///
    /// Tap-to-cursor mapping requires an explicit [`font`](Self::font) (the
    /// theme is not reachable during touch dispatch); without one, taps are
    /// silently ignored.
    #[must_use]
    pub fn on_tap<F: Fn(usize) -> M + 'a>(mut self, f: F) -> Self {
        self.on_tap = Some(Box::new(f));
        self
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Resolve the effective font for layout/draw.
    fn resolved_font<'t>(&'t self, theme: &'t Theme<'a, C>) -> &'t MonoFont<'a> {
        self.font.unwrap_or(theme.default_font())
    }

    /// Glyph cell width for `font`, never zero (guards division).
    fn glyph_w(font: &MonoFont<'_>) -> u32 {
        font.character_size.width.max(1)
    }

    /// Glyph cell height for `font`, never zero.
    fn glyph_h(font: &MonoFont<'_>) -> u32 {
        font.character_size.height.max(1)
    }

    /// Maximum number of chars that fit across the available width.
    fn cols(&self, font: &MonoFont<'_>) -> usize {
        let w = self.rect.size.width;
        (w / Self::glyph_w(font)).max(1) as usize
    }

    /// Lay the source text out into visual lines: split on `'\n'`, then
    /// char-wrap each paragraph to `cols`. Always yields at least one line
    /// (possibly empty) so the cursor and placeholder have somewhere to sit.
    fn layout_lines(&self, font: &MonoFont<'_>) -> Vec<VisualLine> {
        let cols = self.cols(font);
        let mut lines = Vec::new();

        // Walk the text char by char, tracking byte and char offsets, and
        // emit a visual line whenever we hit a newline or run out of columns.
        let mut line_byte_start = 0usize;
        let mut line_char_start = 0usize;
        let mut col = 0usize;
        let mut last_byte = 0usize;

        for (byte_idx, ch) in self.text.char_indices() {
            last_byte = byte_idx + ch.len_utf8();
            if ch == '\n' {
                lines.push(VisualLine {
                    start: line_byte_start,
                    end: byte_idx,
                    char_start: line_char_start,
                    char_len: col,
                });
                line_byte_start = byte_idx + 1;
                line_char_start += col + 1; // +1 for the consumed newline.
                col = 0;
                continue;
            }
            if col >= cols {
                // Char-wrap: close the current line just before this char.
                lines.push(VisualLine {
                    start: line_byte_start,
                    end: byte_idx,
                    char_start: line_char_start,
                    char_len: col,
                });
                line_byte_start = byte_idx;
                line_char_start += col;
                col = 0;
            }
            col += 1;
        }

        // Final (or only) line — the tail after the last newline/wrap.
        lines.push(VisualLine {
            start: line_byte_start,
            end: last_byte.max(line_byte_start),
            char_start: line_char_start,
            char_len: col,
        });

        lines
    }

    /// Locate the cursor as a (line index, column) pair within `lines`.
    /// The cursor is a char index; it belongs to the line whose
    /// `char_start..=char_start+char_len` contains it, biased to the *end*
    /// of a wrapped line so a cursor at a wrap boundary shows on the line it
    /// was typed into.
    fn cursor_line_col(&self, lines: &[VisualLine], cursor_chars: usize) -> (usize, usize) {
        for (i, line) in lines.iter().enumerate() {
            let line_end_char = line.char_start + line.char_len;
            let is_last = i + 1 == lines.len();
            // A cursor exactly at the end of a non-last line that was *wrapped*
            // (not newline-terminated) should still render on this line.
            if cursor_chars < line_end_char || (cursor_chars == line_end_char && is_last) {
                return (i, cursor_chars.saturating_sub(line.char_start));
            }
            // Cursor sits right at a hard line break or wrap end: if the next
            // line starts at the same char (a wrap), keep it here.
            if cursor_chars == line_end_char {
                if let Some(next) = lines.get(i + 1) {
                    if next.char_start == line_end_char {
                        // Wrapped boundary: place at end of this line.
                        return (i, line.char_len);
                    }
                }
            }
        }
        // Past the end: end of the last line.
        let last = lines.len().saturating_sub(1);
        let col = lines.last().map_or(0, |l| l.char_len);
        (last, col)
    }

    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }

    /// Map a touched point to the nearest char index, using the same first
    /// visible line as the draw pass.
    fn index_at(&self, point: Point, font: &MonoFont<'_>) -> usize {
        let lines = self.layout_lines(font);
        if lines.is_empty() {
            return 0;
        }
        let gh = Self::glyph_h(font) as i32;
        let gw = Self::glyph_w(font) as i32;
        let first_line = self.first_visible_line(&lines, font);

        // Which visible row was tapped → absolute line index.
        let rel_y = (point.y - self.rect.top_left.y).max(0);
        let visible_row = (rel_y / gh) as usize;
        let line_idx = (first_line + visible_row).min(lines.len() - 1);
        let line = lines[line_idx];

        // Column from x, rounded to the nearest char boundary, clamped to the
        // line's char length.
        let rel_x = (point.x - self.rect.top_left.x).max(0);
        let col = ((rel_x + gw / 2) / gw) as usize;
        let col = col.min(line.char_len);
        line.char_start + col
    }

    /// Index of the first line to draw so the cursor's line stays visible.
    fn first_visible_line(&self, lines: &[VisualLine], font: &MonoFont<'_>) -> usize {
        let gh = Self::glyph_h(font);
        let rows = (self.rect.size.height / gh).max(1) as usize;
        let cursor_chars = self.cursor.min(self.text.chars().count());
        let (cursor_line, _) = self.cursor_line_col(lines, cursor_chars);
        // Keep the cursor line within the bottom of the viewport.
        if cursor_line + 1 > rows {
            cursor_line + 1 - rows
        } else {
            0
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for TextArea<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(INTRINSIC_W, constraints.max.width);
        let h = self.height.resolve(INTRINSIC_H, constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        let on_tap = self.on_tap.as_ref()?;
        if phase != TouchPhase::Down || !self.hit_test(point) {
            return None;
        }
        // Mapping a tap to a char index requires the font's glyph metrics.
        // `handle_touch` has no access to the theme (that arrives only at
        // `draw`), so tap-to-cursor needs an explicit [`font`](Self::font).
        // Without one the tap is ignored — set `.font(...)` to enable it
        // (the example does). The font is identical to the one used at draw.
        let font = self.font?;
        Some(on_tap(self.index_at(point, font)))
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let font = self.resolved_font(theme);
        let gh = Self::glyph_h(font) as i32;
        let gw = Self::glyph_w(font) as i32;
        let text_color = self.color.unwrap_or(theme.background.on_base);
        let cursor_color = self.cursor_color.unwrap_or(theme.accent.base);

        // Clip everything to the widget rect.
        renderer.push_clip(self.rect);

        let x0 = self.rect.top_left.x;
        let y0 = self.rect.top_left.y;

        // Empty text → dimmed placeholder + a cursor at the start.
        if self.text.is_empty() {
            if !self.placeholder.is_empty() {
                let ph_color = self.placeholder_color.unwrap_or(theme.background.divider);
                renderer.draw_text(
                    &self.placeholder,
                    Point::new(x0, y0 + gh),
                    font,
                    ph_color,
                    Alignment::Left,
                )?;
            }
            let cursor_rect = Rectangle::new(
                Point::new(x0, y0 + 2),
                Size::new(CURSOR_W, gh.max(1) as u32),
            );
            renderer.fill_rect(cursor_rect, cursor_color)?;
            renderer.pop_clip();
            return Ok(());
        }

        let lines = self.layout_lines(font);
        let rows = (self.rect.size.height / Self::glyph_h(font)).max(1) as usize;
        let first_line = self.first_visible_line(&lines, font);

        // Draw the visible window of lines. Baseline is one glyph height
        // below the top of each row (matching `Text`'s top alignment).
        for (row, line) in lines.iter().enumerate().skip(first_line).take(rows) {
            let slice = &self.text[line.start..line.end];
            if !slice.is_empty() {
                let draw_y = y0 + (row - first_line) as i32 * gh + gh;
                renderer.draw_text(
                    slice,
                    Point::new(x0, draw_y),
                    font,
                    text_color,
                    Alignment::Left,
                )?;
            }
        }

        // Draw the cursor bar at (line, col).
        let cursor_chars = self.cursor.min(self.text.chars().count());
        let (cursor_line, cursor_col) = self.cursor_line_col(&lines, cursor_chars);
        if cursor_line >= first_line && cursor_line < first_line + rows {
            let cx = x0 + cursor_col as i32 * gw;
            let cy = y0 + (cursor_line - first_line) as i32 * gh;
            let cursor_rect = Rectangle::new(
                Point::new(cx, cy + 2),
                Size::new(CURSOR_W, gh.max(1) as u32),
            );
            renderer.fill_rect(cursor_rect, cursor_color)?;
        }

        renderer.pop_clip();
        Ok(())
    }
}
