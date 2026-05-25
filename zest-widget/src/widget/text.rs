//! Static text widget. Renders a `Cow<'a, str>` with a font + color.

use alloc::borrow::Cow;
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle,
    text::Alignment as EgAlignment,
};
use zest_core::{Constraints, Horizontal, Length, RenderError, Renderer, TouchPhase, Vertical};
use zest_theme::Theme;

use super::Widget;

/// Static text. Renders `content` using the theme's default font and
/// foreground color unless overridden.
pub struct Text<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    content: Cow<'a, str>,
    align_x: Horizontal,
    align_y: Vertical,
    color: Option<C>,
    font: Option<&'a MonoFont<'a>>,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor, M: Clone> Text<'a, C, M> {
    /// New text widget. Position via `arrange` (or parent container).
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            rect: Rectangle::zero(),
            content: content.into(),
            align_x: Horizontal::Left,
            align_y: Vertical::Center,
            color: None,
            font: None,
            width: Length::Fill,
            height: Length::Fill,
            _phantom: PhantomData,
        }
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

    /// Horizontal alignment within the arranged rect.
    #[must_use]
    pub fn align_x(mut self, h: Horizontal) -> Self {
        self.align_x = h;
        self
    }

    /// Vertical alignment within the arranged rect.
    #[must_use]
    pub fn align_y(mut self, v: Vertical) -> Self {
        self.align_y = v;
        self
    }

    /// Override foreground color (default: `theme.background.on_base`).
    #[must_use]
    pub fn color(mut self, c: C) -> Self {
        self.color = Some(c);
        self
    }

    /// Override font (default: `theme.default_font`).
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Text<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let intrinsic_w = self.font.map_or(0, |f| {
            f.character_size
                .width
                .saturating_mul(self.content.chars().count() as u32)
        });
        let intrinsic_h = self.font.map_or(0, |f| f.character_size.height);
        let w = self.width.resolve(intrinsic_w, constraints.max.width);
        let h = self.height.resolve(intrinsic_h, constraints.max.height);
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
        let color = self.color.unwrap_or(theme.background.on_base);
        let font = self.font.unwrap_or(theme.default_font());

        let (eg_align, x_anchor) = match self.align_x {
            Horizontal::Left => (EgAlignment::Left, self.rect.top_left.x),
            Horizontal::Center => (
                EgAlignment::Center,
                self.rect.top_left.x + (self.rect.size.width / 2) as i32,
            ),
            Horizontal::Right => (
                EgAlignment::Right,
                self.rect.top_left.x + self.rect.size.width as i32,
            ),
        };
        let glyph_h = font.character_size.height as i32;
        let baseline_y = match self.align_y {
            Vertical::Top => self.rect.top_left.y + glyph_h,
            Vertical::Center => {
                self.rect.top_left.y + (self.rect.size.height as i32) / 2 + glyph_h / 3
            }
            Vertical::Bottom => self.rect.top_left.y + self.rect.size.height as i32,
        };

        renderer.draw_text(
            &self.content,
            Point::new(x_anchor, baseline_y),
            font,
            color,
            eg_align,
        )?;
        Ok(())
    }
}
