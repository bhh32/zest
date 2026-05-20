//! Equal-cell grid container. Children fill cells in row-major order.

use super::{Widget, element::Element};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Grid of equal-size cells. Children are placed in row-major order.
pub struct Grid<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    children: Vec<Element<'a, C, M>>,
    cols: u32,
    rows: u32,
    spacing: u32,
    width: Length,
    height: Length,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Grid<'a, C, M> {
    /// Create a `cols x rows` grid. Position and size are assigned by
    /// the parent via `arrange`.
    pub fn new(cols: u32, rows: u32) -> Self {
        Self {
            rect: Rectangle::zero(),
            children: Vec::new(),
            cols,
            rows,
            spacing: 0,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Builder: gap between cells.
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

    /// Builder: add a child. Excess children beyond `cols × rows` are
    /// retained but not laid out.
    #[must_use]
    pub fn push<W>(mut self, child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        self.children.push(Element::new(child));
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

            child.arrange(Rectangle::new(
                Point::new(x, y),
                Size::new(cell_width, cell_height),
            ));
        }
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Grid<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
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
