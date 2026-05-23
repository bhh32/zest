//! Passive multi-series chart. Plots one or more borrowed integer series
//! as line and/or bar graphs, auto-scaling the Y axis to the combined data
//! range and fitting everything inside the arranged rect.
//!
//! Drawing uses only the existing renderer primitives:
//! [`stroke_line`](zest_core::Renderer::stroke_line) for line series, axes,
//! and gridlines; [`fill_rect`](zest_core::Renderer::fill_rect) for bars;
//! and [`fill_circle`](zest_core::Renderer::fill_circle) for optional data
//! point markers.
//!
//! Series data is borrowed (`&'a [i32]`) so the owner keeps it alive for
//! the frame. The API is intentionally small:
//!
//! ```ignore
//! Chart::new()
//!     .line_series(&temps)
//!     .bar_series(&counts)
//!     .axes(true)
//!     .gridlines(4)
//!     .points(true)
//! ```

use super::Widget;
use alloc::vec::Vec;
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// How a single series is rendered.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SeriesKind {
    /// Connected line through the data points.
    Line,
    /// Vertical bar per data point.
    Bar,
}

/// One plotted data series.
struct Series<'a, C> {
    kind: SeriesKind,
    data: &'a [i32],
    /// Explicit color; `None` falls back to a theme color at draw time.
    color: Option<C>,
}

/// A line/bar chart over one or more borrowed integer series.
pub struct Chart<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    series: Vec<Series<'a, C>>,
    axes: bool,
    gridlines: u32,
    points: bool,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<'a, C: PixelColor + 'a, M: Clone> Chart<'a, C, M> {
    /// Create a new empty chart. Add series with [`Self::line_series`] /
    /// [`Self::bar_series`]. Position and size are assigned by the parent
    /// via `arrange`.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            series: Vec::new(),
            axes: false,
            gridlines: 0,
            points: false,
            width: Length::Fill,
            height: Length::Fill,
            _phantom: PhantomData,
        }
    }

    /// Builder: add a line series. Color defaults to `theme.accent.base`.
    #[must_use]
    pub fn line_series(mut self, data: &'a [i32]) -> Self {
        self.series.push(Series {
            kind: SeriesKind::Line,
            data,
            color: None,
        });
        self
    }

    /// Builder: add a line series with an explicit color.
    #[must_use]
    pub fn line_series_colored(mut self, data: &'a [i32], color: C) -> Self {
        self.series.push(Series {
            kind: SeriesKind::Line,
            data,
            color: Some(color),
        });
        self
    }

    /// Builder: add a bar series. Color defaults to `theme.accent.base`.
    #[must_use]
    pub fn bar_series(mut self, data: &'a [i32]) -> Self {
        self.series.push(Series {
            kind: SeriesKind::Bar,
            data,
            color: None,
        });
        self
    }

    /// Builder: add a bar series with an explicit color.
    #[must_use]
    pub fn bar_series_colored(mut self, data: &'a [i32], color: C) -> Self {
        self.series.push(Series {
            kind: SeriesKind::Bar,
            data,
            color: Some(color),
        });
        self
    }

    /// Builder: draw left/bottom axis lines (default: off).
    #[must_use]
    pub fn axes(mut self, axes: bool) -> Self {
        self.axes = axes;
        self
    }

    /// Builder: number of horizontal gridlines to draw (default: 0).
    #[must_use]
    pub fn gridlines(mut self, count: u32) -> Self {
        self.gridlines = count;
        self
    }

    /// Builder: draw a small marker at each line-series data point
    /// (default: off).
    #[must_use]
    pub fn points(mut self, points: bool) -> Self {
        self.points = points;
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

    /// Combined `(min, max)` across all series, or `None` if there is no
    /// data. A flat range is widened by 1 so the divisor is never zero.
    fn data_range(&self) -> Option<(i32, i32)> {
        let mut min = i32::MAX;
        let mut max = i32::MIN;
        let mut any = false;
        for s in &self.series {
            for &v in s.data {
                any = true;
                min = min.min(v);
                max = max.max(v);
            }
        }
        if !any {
            return None;
        }
        if min == max {
            max = min + 1;
        }
        Some((min, max))
    }

    /// Map a data value to a screen Y inside `[top, bottom]` (inverted so
    /// larger values sit higher on screen).
    fn map_y(value: i32, min: i32, max: i32, top: i32, bottom: i32) -> i32 {
        let span = (max - min) as i64;
        let frac = ((value - min) as i64 * (bottom - top) as i64) / span;
        bottom - frac as i32
    }

    /// Map a data index to a screen X inside `[left, right]`. With a single
    /// point everything collapses to `left`.
    fn map_x(index: usize, count: usize, left: i32, right: i32) -> i32 {
        if count <= 1 {
            return left;
        }
        left + ((index as i64 * (right - left) as i64) / (count as i64 - 1)) as i32
    }
}

impl<'a, C: PixelColor + 'a, M: Clone> Default for Chart<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone> Widget<C, M> for Chart<'a, C, M> {
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
        // A 1px inset keeps strokes off the very edge of the slot.
        let left = self.rect.top_left.x + 1;
        let right = self.rect.top_left.x + self.rect.size.width as i32 - 2;
        let top = self.rect.top_left.y + 1;
        let bottom = self.rect.top_left.y + self.rect.size.height as i32 - 2;
        if right <= left || bottom <= top {
            return Ok(());
        }

        let grid = theme.background.divider;

        // Horizontal gridlines, evenly spaced across the plot height.
        if self.gridlines > 0 {
            let n = self.gridlines;
            for i in 0..=n {
                let y = top + ((i as i64 * (bottom - top) as i64) / n as i64) as i32;
                renderer.stroke_line(Point::new(left, y), Point::new(right, y), grid, 1)?;
            }
        }

        // Axes (left + bottom).
        if self.axes {
            let axis = theme.background.on_base;
            renderer.stroke_line(Point::new(left, top), Point::new(left, bottom), axis, 1)?;
            renderer.stroke_line(Point::new(left, bottom), Point::new(right, bottom), axis, 1)?;
        }

        let Some((min, max)) = self.data_range() else {
            return Ok(());
        };

        // Count bar series so multiple bar series can share each x-slot.
        let bar_series_count = self
            .series
            .iter()
            .filter(|s| s.kind == SeriesKind::Bar)
            .count()
            .max(1);
        let mut bar_index = 0usize;

        for s in &self.series {
            if s.data.is_empty() {
                continue;
            }
            let color = s.color.unwrap_or(theme.accent.base);
            let count = s.data.len();
            match s.kind {
                SeriesKind::Line => {
                    let mut prev: Option<Point> = None;
                    for (i, &v) in s.data.iter().enumerate() {
                        let x = Self::map_x(i, count, left, right);
                        let y = Self::map_y(v, min, max, top, bottom);
                        let p = Point::new(x, y);
                        if let Some(prev) = prev {
                            renderer.stroke_line(prev, p, color, 1)?;
                        }
                        if self.points {
                            renderer.fill_circle(p, 2, color)?;
                        }
                        prev = Some(p);
                    }
                }
                SeriesKind::Bar => {
                    // Slot width per data point; bar series split each slot.
                    let slot = ((right - left) / count.max(1) as i32).max(1);
                    let group_w = (slot * 4 / 5).max(1);
                    let bar_w = (group_w / bar_series_count as i32).max(1);
                    for (i, &v) in s.data.iter().enumerate() {
                        let slot_left = left + i as i32 * slot;
                        let bx = slot_left + bar_index as i32 * bar_w;
                        let y = Self::map_y(v, min, max, top, bottom);
                        let h = (bottom - y).max(0) as u32;
                        if h == 0 {
                            continue;
                        }
                        renderer.fill_rect(
                            Rectangle::new(Point::new(bx, y), Size::new(bar_w as u32, h)),
                            color,
                        )?;
                    }
                    bar_index += 1;
                }
            }
        }
        Ok(())
    }
}
