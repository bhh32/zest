//! Vertical layout container. Holds children as `Vec<Element>`, runs a
//! flex resolve modeled on iced's `core/src/layout/flex.rs`:
//!
//! 1. allocate `Length::Fixed` slots,
//! 2. measure `Length::Shrink` children with the residual constraint,
//! 3. divide what's left across `Length::Fill` / `Length::FillPortion`
//!    children proportional to their portion weights.

use super::{Widget, element::Element};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Vertical stack of widgets.
pub struct Column<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    spacing: u32,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Column<'a, C, M> {
    /// Create a new empty column. Position and size are assigned by
    /// the parent via `arrange`.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            children: Vec::new(),
            spacing: 2,
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
        let avail_w = self.rect.size.width;
        let total_spacing = self.spacing.saturating_mul(n as u32 - 1);
        let avail_h = self.rect.size.height.saturating_sub(total_spacing);

        // Pass 1 + 2: collect per-child intent + height.
        let cross = Constraints::loose(Size::new(avail_w, avail_h));
        let mut heights: Vec<u32> = Vec::with_capacity(n);
        let mut fixed_total: u32 = 0;
        let mut shrink_total: u32 = 0;
        let mut portion_total: u32 = 0;

        for child in &mut self.children {
            let (_, h_intent) = child.preferred_size();
            let h = match h_intent {
                Length::Fixed(px) => {
                    fixed_total = fixed_total.saturating_add(px);
                    px
                }
                Length::Shrink => {
                    let m = child.measure(cross).height;
                    shrink_total = shrink_total.saturating_add(m);
                    m
                }
                Length::Fill | Length::FillPortion(_) => {
                    portion_total = portion_total.saturating_add(h_intent.portion());
                    0
                }
            };
            heights.push(h);
        }

        // Pass 3: distribute remaining height to Fill / FillPortion.
        let consumed = fixed_total.saturating_add(shrink_total);
        let remaining = avail_h.saturating_sub(consumed);
        if portion_total > 0 {
            let unit = remaining / portion_total;
            for (child, h) in self.children.iter_mut().zip(heights.iter_mut()) {
                let (_, h_intent) = child.preferred_size();
                if h_intent == Length::Fill || matches!(h_intent, Length::FillPortion(_)) {
                    *h = unit.saturating_mul(h_intent.portion());
                }
            }
        }

        // Pass 4: arrange.
        let mut y = self.rect.top_left.y;
        for (child, h) in self.children.iter_mut().zip(heights.iter()) {
            let cell = Rectangle::new(
                Point::new(self.rect.top_left.x, y),
                Size::new(avail_w, *h),
            );
            child.arrange(cell);
            y += *h as i32 + self.spacing as i32;
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Column<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Column<'a, C, M> {
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
