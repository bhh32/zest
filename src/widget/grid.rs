//! Equal-cell grid container. Children fill cells in row-major order.

use crate::{Element, Renderer, Theme, Widget, renderer::RenderError};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};

/// Grid of equal-size cells. Children are placed in row-major order.
pub struct Grid<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    cols: u32,
    rows: u32,
    spacing: u32,
}

impl<'a, C: PixelColor, M: Clone> Grid<'a, C, M> {
    /// Create a `cols x rows` grid at `rect`.
    pub fn new(rect: Rectangle, cols: u32, rows: u32) -> Self {
        Self {
            rect,
            children: Vec::new(),
            cols,
            rows,
            spacing: 0,
        }
    }

    /// Builder: gap between cells.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Builder: add a child. Excess children beyond `cols x rows` are
    /// retained but not laid out.
    pub fn push(mut self, child: impl Into<Element<'a, C, M>>) -> Self {
        self.children.push(child.into());
        self
    }

    fn relayout(&mut self) {
        if self.cols == 0 || self.rows == 0 {
            return;
        }

        let h_spacing = self.spacing * (self.cols.saturating_sub(1));
        let v_spacing = self.spacing * (self.rows.saturating_sub(1));
        let cell_width = self.rect.size.width.saturating_sub(h_spacing) / self.cols;
        let cell_height = self.rect.size.height.saturating_sub(v_spacing) / self.rows;

        for (idx, child) in self.children.iter_mut().enumerate() {
            let row = (idx as u32) / self.cols;
            let col = (idx as u32) % self.cols;

            if row >= self.rows {
                break;
            }

            let x = self.rect.top_left.x + (col * (cell_width + self.spacing)) as i32;
            let y = self.rect.top_left.y + (row * (cell_height + self.spacing)) as i32;

            child.set_rect(Rectangle::new(
                Point::new(x, y),
                Size::new(cell_width, cell_height),
            ));
        }
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<'a, C, M> for Grid<'a, C, M> {
    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn set_rect(&mut self, rect: Rectangle) {
        self.rect = rect;
        self.relayout();
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
