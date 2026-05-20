//! Horizontal layout container. Mirror of `Column` on the width axis.
//! Children declare their slot intent via `.width(Length::...)`; the
//! per-child `weight` parameter that previously lived on `push_weighted`
//! is now expressed as `child.width(Length::FillPortion(n))`.

use super::{Widget, element::Element};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Horizontal stack of widgets.
pub struct Row<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    spacing: u32,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Row<'a, C, M> {
    /// Create a new empty row. Position and size are assigned by the
    /// parent via `arrange`.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            children: Vec::new(),
            spacing: 0,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Builder: gap between children.
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
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

    /// Builder: add a child.
    #[must_use]
    pub fn push<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.children.push(Element::new(child));
        self
    }

    fn relayout(&mut self) {
        let n = self.children.len();
        if n == 0 {
            return;
        }
        let avail_h = self.rect.size.height;
        let total_spacing = self.spacing.saturating_mul(n as u32 - 1);
        let avail_w = self.rect.size.width.saturating_sub(total_spacing);

        let cross = Constraints::loose(Size::new(avail_w, avail_h));
        let mut widths: Vec<u32> = Vec::with_capacity(n);
        let mut fixed_total: u32 = 0;
        let mut shrink_total: u32 = 0;
        let mut portion_total: u32 = 0;

        for child in &mut self.children {
            let (w_intent, _) = child.preferred_size();
            let w = match w_intent {
                Length::Fixed(px) => {
                    fixed_total = fixed_total.saturating_add(px);
                    px
                }
                Length::Shrink => {
                    let m = child.measure(cross).width;
                    shrink_total = shrink_total.saturating_add(m);
                    m
                }
                Length::Fill | Length::FillPortion(_) => {
                    portion_total = portion_total.saturating_add(w_intent.portion());
                    0
                }
            };
            widths.push(w);
        }

        let consumed = fixed_total.saturating_add(shrink_total);
        let remaining = avail_w.saturating_sub(consumed);
        if portion_total > 0 {
            let unit = remaining / portion_total;
            for (child, w) in self.children.iter_mut().zip(widths.iter_mut()) {
                let (w_intent, _) = child.preferred_size();
                if w_intent == Length::Fill || matches!(w_intent, Length::FillPortion(_)) {
                    *w = unit.saturating_mul(w_intent.portion());
                }
            }
        }

        let mut x = self.rect.top_left.x;
        for (child, w) in self.children.iter_mut().zip(widths.iter()) {
            let cell = Rectangle::new(
                Point::new(x, self.rect.top_left.y),
                Size::new(*w, avail_h),
            );
            child.arrange(cell);
            x += *w as i32 + self.spacing as i32;
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Row<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Row<'a, C, M> {
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
        self.relayout();
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        for child in self.children.iter_mut().rev() {
            if let Some(msg) = child.handle_touch(point, phase) {
                return Some(msg);
            }
        }
        None
    }

    fn mark_pressed(&mut self, point: Point) {
        for child in &mut self.children {
            child.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        for child in &self.children {
            child.draw(renderer, theme)?;
        }
        Ok(())
    }
}
