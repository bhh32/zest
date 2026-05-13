//! Vertical layout container. Holds children as `Vec<Element>`, lays them
//! out from top to bottom with optional spacing.

use crate::{Element, Renderer, Theme, Widget, renderer::RenderError};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};

/// Vertical stack of widgets.
pub struct Column<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    spacing: u32,
}

impl<'a, C: PixelColor, M: Clone> Column<'a, C, M> {
    /// Create a new emtpy column at `rect`.
    pub fn new(rect: Rectangle) -> Self {
        Self {
            rect,
            children: Vec::new(),
            spacing: 2,
        }
    }

    /// Builder: set teh gap between children.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Builder: add a child. Children are laid out top-to-bottom with equal
    /// height (`rect.height - total_spacing) / count`.
    pub fn push(mut self, child: impl Into<Element<'a, C, M>>) -> Self {
        self.children.push(child.into());
        self
    }

    fn relayout(&mut self) {
        let child_count = self.children.len();

        if child_count == 0 {
            return;
        }

        let total_spacing = self.spacing * (child_count as u32 - 1);
        let total_height = self.rect.size.height.saturating_sub(total_spacing);
        let child_height = total_height / child_count as u32;
        let mut y = self.rect.top_left.y;

        for child in &mut self.children {
            let rect = Rectangle::new(
                Point::new(self.rect.top_left.x, y),
                Size::new(self.rect.size.width, child_height),
            );

            child.set_rect(rect);
            y += child_height as i32 + self.spacing as i32;
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<'a, C, M> for Column<'a, C, M> {
    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn set_rect(&mut self, rect: Rectangle) {
        self.rect = rect;
        self.relayout();
    }

    fn handle_touch(&mut self, point: Point) -> Option<M> {
        // Lazy ensure layout is current. Cheap when nothing changed.
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
