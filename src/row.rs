//! Horizontal layout container.

use crate::{Element, Renderer, Theme, Widget, renderer::RenderError};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};

/// Horizontal stack of widgets.
pub struct Row<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    weights: Vec<u32>,
    spacing: u32,
}

impl<'a, C: PixelColor, M: Clone> Row<'a, C, M> {
    /// Create a new empty row at `rect`.
    pub fn new(rect: Rectangle) -> Self {
        Self {
            rect,
            children: Vec::new(),
            weights: Vec::new(),
            spacing: 0,
        }
    }

    /// Builder: gap between children.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Builder: add a child.
    pub fn push(mut self, child: impl Into<Element<'a, C, M>>) -> Self {
        self.children.push(child.into());
        self.weights.push(1);
        self
    }

    /// Push a child with explicit flex weight. Width = avaliable * (weight / total_weight).
    pub fn push_weighted(mut self, child: impl Into<Element<'a, C, M>>, weight: u32) -> Self {
        self.children.push(child.into());
        self.weights.push(weight.max(1));
        self
    }

    fn relayout(&mut self) {
        let child_count = self.children.len();

        if child_count == 0 {
            return;
        }

        let total_spacing = self.spacing * (child_count as u32 - 1);
        let total_width = self.rect.size.width.saturating_sub(total_spacing);
        let total_weight: u32 = self.weights.iter().sum();

        if total_weight == 0 {
            return;
        }

        let mut x = self.rect.top_left.x;

        for (child, &weight) in self.children.iter_mut().zip(self.weights.iter()) {
            let child_width = total_width * weight / total_weight;
            let rect = Rectangle::new(
                Point::new(x, self.rect.top_left.y),
                Size::new(child_width, self.rect.size.height),
            );
            child.set_rect(rect);
            x += child_width as i32 + self.spacing as i32;
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<'a, C, M> for Row<'a, C, M> {
    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn set_rect(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn handle_touch(&mut self, point: Point) -> Option<M> {
        if !self.children.is_empty() && self.children[0].rect().size.width == 0 {
            self.relayout();
        }

        for child in self.children.iter_mut().rev() {
            if let Some(msg) = child.handle_touch(point) {
                return Some(msg);
            }
        }

        None
    }

    fn draw(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'a, C>,
    ) -> Result<(), RenderError> {
        for child in &self.children {
            child.draw(renderer, theme)?;
        }

        Ok(())
    }
}
