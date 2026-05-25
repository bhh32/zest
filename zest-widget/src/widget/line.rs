//! Passive polyline. Connects a borrowed slice of points with straight
//! segments via [`Renderer::stroke_line`](zest_core::Renderer::stroke_line).
//!
//! The point slice is borrowed (`&'a [Point]`) so the owner — typically a
//! screen holding the data — keeps it alive for the frame. By default the
//! points are interpreted **content-relative**: each point is offset by the
//! widget's arranged top-left. Call [`Line::absolute`] to treat the points
//! as already being in screen coordinates.

use super::Widget;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// A multi-segment line through a borrowed point slice.
pub struct Line<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    points: &'a [Point],
    color: Option<C>,
    line_width: u32,
    /// When `true`, points are screen-absolute; otherwise content-relative
    /// to the arranged rect's top-left.
    absolute: bool,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor, M: Clone> Line<'a, C, M> {
    /// New polyline through `points` (content-relative by default).
    /// Position and size are assigned by the parent via `arrange`.
    pub fn new(points: &'a [Point]) -> Self {
        Self {
            rect: Rectangle::zero(),
            points,
            color: None,
            line_width: 1,
            absolute: false,
            width: Length::Fill,
            height: Length::Fill,
            _phantom: PhantomData,
        }
    }

    /// Stroke color (default: `theme.background.on_base`).
    #[must_use]
    pub fn color(mut self, color: C) -> Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in pixels (default: 1).
    #[must_use]
    pub fn width_px(mut self, width: u32) -> Self {
        self.line_width = width;
        self
    }

    /// Interpret the points as screen-absolute coordinates
    /// instead of content-relative to the arranged rect.
    #[must_use]
    pub fn absolute(mut self) -> Self {
        self.absolute = true;
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
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Line<'a, C, M> {
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
        if self.points.len() < 2 {
            return Ok(());
        }
        let color = self.color.unwrap_or(theme.background.on_base);
        let offset = if self.absolute {
            Point::zero()
        } else {
            self.rect.top_left
        };
        for pair in self.points.windows(2) {
            renderer.stroke_line(pair[0] + offset, pair[1] + offset, color, self.line_width)?;
        }
        Ok(())
    }
}
