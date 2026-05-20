//! Transparent filler widget. Used to push siblings to one end of a
//! [`Row`](super::row::Row) / [`Column`](super::column::Column),
//! create explicit fixed gaps, or reserve space.
//!
//! Construct via the free functions [`horizontal_space`] / [`vertical_space`]
//! for the common cases, or build a custom [`Space`] directly.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Empty widget that draws nothing. Carries [`Length`] intent on both
/// axes so it slots cleanly into the standard flex layout.
pub struct Space<C: PixelColor, M: Clone> {
    rect: Rectangle,
    width: Length,
    height: Length,
    _phantom: PhantomData<(C, M)>,
}

impl<C: PixelColor, M: Clone> Space<C, M> {
    /// Construct a space with explicit `width` and `height` intents.
    pub fn new(width: impl Into<Length>, height: impl Into<Length>) -> Self {
        Self {
            rect: Rectangle::zero(),
            width: width.into(),
            height: height.into(),
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
}

/// Horizontal filler — fills available width, zero height. Useful as a
/// spacer between [`Row`](super::row::Row) children to push them to
/// opposite ends.
pub fn horizontal_space<C: PixelColor, M: Clone>() -> Space<C, M> {
    Space::new(Length::Fill, Length::Fixed(0))
}

/// Vertical filler — fills available height, zero width. Useful as a
/// spacer between [`Column`](super::column::Column) children to push
/// them to top and bottom.
pub fn vertical_space<C: PixelColor, M: Clone>() -> Space<C, M> {
    Space::new(Length::Fixed(0), Length::Fill)
}

impl<C: PixelColor, M: Clone> Widget<C, M> for Space<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(0, constraints.max.width);
        let h = self.height.resolve(0, constraints.max.height);
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
        _renderer: &mut dyn Renderer<C>,
        _theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        Ok(())
    }
}
