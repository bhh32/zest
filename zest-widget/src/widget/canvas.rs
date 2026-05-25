//! Drawable surface: an owned [`CanvasBuffer`] the host mutates plus a
//! passive [`Canvas`] widget that blits it to the screen.
//!
//! Because widgets are rebuilt every frame, the *mutable* pixel state
//! lives on the host as a [`CanvasBuffer`] — the screen owns it as a
//! field, mutates it in `update`, and lends it to the widget by reference
//! in `view`. The [`Canvas`] widget itself is stateless: it borrows
//! `&'a CanvasBuffer<C>` and draws it via [`Renderer::draw_image`],
//! mirroring [`Image`](super::image::Image).

use super::Widget;
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Owned, row-major pixel buffer with simple drawing operations.
///
/// The host stores one as a field, mutates it in `update`, and passes
/// `&buffer` to a [`Canvas`] widget in `view`.
#[derive(Clone, Debug)]
pub struct CanvasBuffer<C: PixelColor> {
    width: u32,
    height: u32,
    pixels: Vec<C>,
}

impl<C: PixelColor> CanvasBuffer<C> {
    /// Create a `width` x `height` buffer filled with `fill`.
    #[must_use]
    pub fn new(width: u32, height: u32, fill: C) -> Self {
        Self {
            width,
            height,
            pixels: vec![fill; (width as usize) * (height as usize)],
        }
    }

    /// Buffer width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Buffer height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Buffer dimensions as a [`Size`].
    #[must_use]
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Borrow the row-major pixel slice (e.g. for a [`Canvas`] widget).
    #[must_use]
    pub fn pixels(&self) -> &[C] {
        &self.pixels
    }

    /// Overwrite every pixel with `color`.
    pub fn clear(&mut self, color: C) {
        for px in &mut self.pixels {
            *px = color;
        }
    }

    /// Set a single pixel. Out-of-bounds coordinates are ignored.
    pub fn set_pixel(&mut self, x: i32, y: i32, color: C) {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return;
        }
        let idx = (y as u32 * self.width + x as u32) as usize;
        self.pixels[idx] = color;
    }

    /// Fill an axis-aligned rectangle (clipped to the buffer).
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: C) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w as i32).min(self.width as i32);
        let y1 = (y + h as i32).min(self.height as i32);
        let mut py = y0;
        while py < y1 {
            let mut px = x0;
            while px < x1 {
                let idx = (py as u32 * self.width + px as u32) as usize;
                self.pixels[idx] = color;
                px += 1;
            }
            py += 1;
        }
    }

    /// Draw a line from `(x0, y0)` to `(x1, y1)` using Bresenham's
    /// algorithm. Trig- and float-free, so it stays `no_std` friendly.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: C) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

/// Passive widget that blits a borrowed [`CanvasBuffer`] to the screen.
///
/// Defaults to the buffer's intrinsic size; centered when its arranged
/// slot is larger, like [`Image`](super::image::Image).
pub struct Canvas<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    buffer: &'a CanvasBuffer<C>,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor, M: Clone> Canvas<'a, C, M> {
    /// Create a canvas widget that displays `buffer`. Position is
    /// assigned by the parent via `arrange`.
    pub fn new(buffer: &'a CanvasBuffer<C>) -> Self {
        Self {
            rect: Rectangle::zero(),
            buffer,
            width: Length::Shrink,
            height: Length::Shrink,
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
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Canvas<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let size = self.buffer.size();
        let w = self.width.resolve(size.width, constraints.max.width);
        let h = self.height.resolve(size.height, constraints.max.height);
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
        _theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let size = self.buffer.size();
        let dx = (self.rect.size.width as i32 - size.width as i32).max(0) / 2;
        let dy = (self.rect.size.height as i32 - size.height as i32).max(0) / 2;
        let origin = self.rect.top_left + Point::new(dx, dy);
        renderer.draw_image(origin, size, self.buffer.pixels())
    }
}
