//! Thin horizontal or vertical rule. Reads color from
//! `theme.background.divider` unless overridden.
//!
//! Construct via [`horizontal_divider`] / [`vertical_divider`] for
//! the common cases, or build a custom [`Divider`] directly.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// A thin solid rule. `width`/`height` follow the standard [`Length`]
/// rules; the "thin" axis is set via [`Self::thickness`] (which is
/// just a shortcut for `Fixed(n)` on that axis).
pub struct Divider<C: PixelColor, M: Clone> {
    rect: Rectangle,
    width: Length,
    height: Length,
    color: Option<C>,
    _phantom: PhantomData<M>,
}

impl<C: PixelColor, M: Clone> Divider<C, M> {
    /// Construct a divider with explicit `width` and `height` intents.
    pub fn new(width: impl Into<Length>, height: impl Into<Length>) -> Self {
        Self {
            rect: Rectangle::zero(),
            width: width.into(),
            height: height.into(),
            color: None,
            _phantom: PhantomData,
        }
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Builder: explicit color (default: `theme.background.divider`).
    #[must_use]
    pub fn color(mut self, color: C) -> Self {
        self.color = Some(color);
        self
    }

    /// Builder: set the thin-axis thickness. For a horizontal divider
    /// this is the height; for a vertical divider, the width. Chooses
    /// the smaller of the current two axes as the "thin" one.
    #[must_use]
    pub fn thickness(self, thickness: u32) -> Self {
        let thin_is_height = matches!(
            (self.width, self.height),
            (Length::Fill | Length::FillPortion(_), _),
        );
        if thin_is_height {
            self.height(Length::Fixed(thickness))
        } else {
            self.width(Length::Fixed(thickness))
        }
    }
}

/// Horizontal rule — fills container width, 1px tall by default.
pub fn horizontal_divider<C: PixelColor, M: Clone>() -> Divider<C, M> {
    Divider::new(Length::Fill, Length::Fixed(1))
}

/// Vertical rule — fills container height, 1px wide by default.
pub fn vertical_divider<C: PixelColor, M: Clone>() -> Divider<C, M> {
    Divider::new(Length::Fixed(1), Length::Fill)
}

impl<C: PixelColor, M: Clone> Widget<C, M> for Divider<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(1, constraints.max.width);
        let h = self.height.resolve(1, constraints.max.height);
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
        let color = self.color.unwrap_or(theme.background.divider);
        renderer.fill_rect(self.rect, color)?;
        Ok(())
    }
}
