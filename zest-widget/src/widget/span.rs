//! Rich text: a paragraph of independently-styled inline runs.
//!
//! A [`Span`] is one styled run — its text plus an optional color and
//! optional font override. A [`SpanGroup`] is a passive widget holding a
//! `Vec<Span>` that lays the runs out inline, left to right, wrapping to
//! the next line at word boundaries when a run would overflow the
//! arranged width. Each run is painted with its own color and font via
//! [`Renderer::draw_text`].
//!
//! Wrapping is whitespace-based and works per-character for runs without
//! spaces, using the run font's fixed `character_size` (mono fonts only).

use super::Widget;
use alloc::{borrow::Cow, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// One styled run of text within a [`SpanGroup`].
///
/// `color` and `font` are optional overrides; when `None` the
/// [`SpanGroup`] falls back to the theme's `background.on_base` color and
/// default font respectively.
#[derive(Clone)]
pub struct Span<'a, C: PixelColor> {
    /// The run's text.
    pub text: Cow<'a, str>,
    /// Optional color override.
    pub color: Option<C>,
    /// Optional font override.
    pub font: Option<&'a MonoFont<'a>>,
}

impl<'a, C: PixelColor> Span<'a, C> {
    /// Create a plain run with no style overrides (inherits the group's
    /// theme color and default font).
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            color: None,
            font: None,
        }
    }

    /// Override this run's color.
    #[must_use]
    pub fn color(mut self, color: C) -> Self {
        self.color = Some(color);
        self
    }

    /// Override this run's font.
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }
}

/// Passive widget laying out a sequence of [`Span`] runs inline with word
/// wrapping.
pub struct SpanGroup<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    spans: Vec<Span<'a, C>>,
    line_spacing: u32,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor, M: Clone> SpanGroup<'a, C, M> {
    /// Create a new empty span group.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            spans: Vec::new(),
            line_spacing: 2,
            width: Length::Fill,
            height: Length::Fill,
            _phantom: PhantomData,
        }
    }

    /// Append a fully-built [`Span`].
    #[must_use]
    pub fn push(mut self, span: Span<'a, C>) -> Self {
        self.spans.push(span);
        self
    }

    /// Append a plain run by text. Chain `.color(..)` / `.font(..)`
    /// via [`Span`] when more control is needed, or use [`SpanGroup::push`].
    #[must_use]
    pub fn span(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.spans.push(Span::new(text));
        self
    }

    /// Extra vertical gap between wrapped lines, in pixels.
    #[must_use]
    pub fn line_spacing(mut self, spacing: u32) -> Self {
        self.line_spacing = spacing;
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

    /// The tallest run font height, used as the line height. Falls back to
    /// `fallback` (the theme default font height) when there are no runs
    /// with an explicit font.
    fn line_height(&self, fallback: u32) -> u32 {
        let mut h = fallback;
        for span in &self.spans {
            if let Some(font) = span.font {
                h = h.max(font.character_size.height);
            }
        }
        h
    }
}

impl<'a, C: PixelColor, M: Clone> Default for SpanGroup<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for SpanGroup<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self
            .height
            .resolve(constraints.max.height, constraints.max.height);
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

    fn handle_touch(&mut self, _point: Point, _phase: TouchPhase) -> Option<M> {
        None
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let default_font = theme.default_font();
        let default_color = theme.background.on_base;
        let line_h = self.line_height(default_font.character_size.height);

        let left = self.rect.top_left.x;
        let right = self.rect.top_left.x + self.rect.size.width as i32;
        let bottom = self.rect.top_left.y + self.rect.size.height as i32;

        // Pen position tracks the top-left of the next glyph cell.
        let mut pen_x = left;
        let mut pen_y = self.rect.top_left.y;

        for span in &self.spans {
            let font = span.font.unwrap_or(default_font);
            let color = span.color.unwrap_or(default_color);
            let glyph_w = font.character_size.width as i32;
            let glyph_h = font.character_size.height as i32;

            // Split the run into whitespace-preserving words so wrapping
            // breaks at spaces. A leading word may continue the current
            // line; subsequent words wrap as needed.
            for word in split_keep_spaces(&span.text) {
                let word_w = word.chars().count() as i32 * glyph_w;
                // Wrap if this word would overflow and we are not at the
                // line start (avoid wrapping a word wider than the line).
                if pen_x > left && pen_x + word_w > right {
                    pen_x = left;
                    pen_y += line_h as i32 + self.line_spacing as i32;
                }
                if pen_y >= bottom {
                    return Ok(());
                }
                // A leading space at the start of a fresh line is dropped
                // so wrapped lines align flush left.
                let to_draw = if pen_x == left {
                    word.trim_start_matches(' ')
                } else {
                    word
                };
                if !to_draw.is_empty() {
                    // draw_text's baseline anchor sits at the glyph
                    // bottom; offset from the cell top.
                    let baseline = pen_y + glyph_h;
                    renderer.draw_text(
                        to_draw,
                        Point::new(pen_x, baseline),
                        font,
                        color,
                        Alignment::Left,
                    )?;
                    pen_x += to_draw.chars().count() as i32 * glyph_w;
                }
            }
        }
        Ok(())
    }
}

/// Split `s` into segments where each space character starts a new
/// segment, preserving the spaces. e.g. `"a b"` → `["a", " b"]`. This
/// lets the layout wrap at word boundaries while keeping inter-word
/// spacing.
fn split_keep_spaces(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' && i > start {
            out.push(&s[start..i]);
            start = i;
        }
        i += 1;
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}
