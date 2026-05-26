//! Vertically scrollable grid of text cells with an optional header row.
//!
//! `Table` lays out borrowed rows of string cells in evenly-sized columns,
//! draws an optional pinned-looking header (styled with accent colors), and
//! scrolls the body via the same engine as [`Column`] — exposing the same
//! scroll surface ([`Table::scrollable`], [`Table::scroll_state`],
//! [`Table::scrollbar`], [`Table::snap`], [`Table::on_scroll`]).
//!
//! ## Data model
//!
//! Cell data is borrowed: a table is built from `&'a [&'a [&'a str]]` via
//! [`Table::rows`], or one row at a time with [`Table::row`] (each row a
//! `&'a [&'a str]`). Columns are sized evenly across the table width. An
//! optional header row is supplied with [`Table::header`].
//!
//! ## Composition
//!
//! The body is a scrollable [`Column`] of [`TableRow`] widgets; the header
//! (when present) is drawn above the scroll viewport and excluded from it, so
//! it stays put while the body scrolls. Each body row can report a tapped
//! `(row, col)` through [`Table::on_select`].
//!
//! ## Colors
//!
//! Header uses `theme.accent` (base background, `on_base` text). Body rows
//! alternate `theme.primary.base` and a subtly shaded variant for readability,
//! a pressed/selected cell uses `theme.accent.pressed`, cell text uses
//! `theme.primary.on_base`, and grid lines use `theme.primary.divider`.

use super::{Widget, column::Column};
use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{
    Constraints, Length, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState,
    ScrollbarMode, SnapMode, TouchPhase, UiAction, WidgetId,
};
use zest_theme::Theme;

/// Default height (px) of a table row (header and body).
pub const TABLE_ROW_HEIGHT: u32 = 32;
/// Horizontal padding (px) inside each cell.
pub const CELL_PADDING_X: u32 = 6;

/// A single table row of borrowed string cells.
///
/// Drawn as evenly-spaced cells with vertical grid lines between columns and
/// a bottom divider. Reports a tapped `(row, col)` through the shared select
/// callback owned by the parent [`Table`]. Press/`mark_pressed` semantics
/// mirror [`Button`](crate::Button) for tap-vs-scroll behavior.
pub struct TableRow<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    /// This row's index within the table body.
    row: usize,
    /// Stable id base for this row's focusable cells.
    base_id: Option<WidgetId>,
    /// Borrowed cell strings, one per column.
    cells: &'a [&'a str],
    /// Total column count (so short rows still align to the grid).
    columns: usize,
    /// Shared select callback owned by the parent table.
    on_select: Option<Rc<dyn Fn(usize, usize) -> M + 'a>>,
    /// Whether this row should render with the alternate (zebra) shade.
    alternate: bool,
    /// `(row, col)` of a host-selected cell to highlight, if it lies here.
    selected_col: Option<usize>,
    /// Column index currently focused.
    focused_col: Option<usize>,
    /// Column index currently pressed (set on Down, cleared on Up/cancel).
    pressed_col: Option<usize>,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> TableRow<'a, C, M> {
    fn new(row: usize, cells: &'a [&'a str], columns: usize) -> Self {
        Self {
            rect: Rectangle::zero(),
            row,
            base_id: None,
            cells,
            columns,
            on_select: None,
            alternate: false,
            selected_col: None,
            focused_col: None,
            pressed_col: None,
            width: Length::Fill,
            height: Length::Fixed(TABLE_ROW_HEIGHT),
            _color: PhantomData,
        }
    }

    fn is_enabled(&self) -> bool {
        self.on_select.is_some()
    }

    /// Width (px) of one column given the row's current width.
    fn col_width(&self) -> u32 {
        let cols = self.columns.max(1) as u32;
        self.rect.size.width / cols
    }

    /// Column index hit by `point`, if `point` is inside the row.
    fn col_at(&self, point: Point) -> Option<usize> {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        if point.x < tl.x || point.x >= br.x || point.y < tl.y || point.y >= br.y {
            return None;
        }
        let col_w = self.col_width().max(1) as i32;
        let col = ((point.x - tl.x) / col_w) as usize;
        Some(col.min(self.columns.saturating_sub(1)))
    }

    fn cell_id(&self, col: usize) -> Option<WidgetId> {
        self.base_id
            .map(|base| WidgetId::new(base.raw().wrapping_add(col as u64)))
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for TableRow<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self
            .height
            .resolve(TABLE_ROW_HEIGHT, constraints.max.height);
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

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        if !self.is_enabled() {
            return None;
        }
        match phase {
            TouchPhase::Down => {
                self.pressed_col = self.col_at(point);
                None
            }
            TouchPhase::Up => {
                let hit = self.col_at(point);
                let fired = match (self.pressed_col, hit) {
                    (Some(p), Some(h)) if p == h => {
                        self.on_select.as_ref().map(|cb| cb(self.row, h))
                    }
                    _ => None,
                };
                self.pressed_col = None;
                fired
            }
            TouchPhase::Moved => {
                if self.col_at(point) != self.pressed_col {
                    self.pressed_col = None;
                }
                None
            }
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.is_enabled() {
            self.pressed_col = self.col_at(point);
        }
    }

    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        if !self.is_enabled() {
            return;
        }
        for col in 0..self.columns {
            if let Some(id) = self.cell_id(col) {
                out.push(id);
            }
        }
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        self.focused_col = focused
            .and_then(|target| (0..self.columns).find(|col| self.cell_id(*col) == Some(target)));
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        let col = (0..self.columns).find(|candidate| self.cell_id(*candidate) == Some(target))?;
        match action {
            UiAction::Activate => self.on_select.as_ref().map(|cb| cb(self.row, col)),
            _ => None,
        }
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        let col = (0..self.columns).find(|candidate| self.cell_id(*candidate) == Some(target))?;
        let col_w = self.col_width();
        Some(Rectangle::new(
            Point::new(
                self.rect.top_left.x + (col_w * col as u32) as i32,
                self.rect.top_left.y,
            ),
            Size::new(col_w, self.rect.size.height),
        ))
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        self.col_at(point).and_then(|col| self.cell_id(col))
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let font = theme.default_font();
        // Row background (zebra striping for readability).
        let bg = if self.alternate {
            theme.secondary.base
        } else {
            theme.primary.base
        };
        renderer.fill_rect(self.rect, bg)?;

        let col_w = self.col_width();
        let glyph_h = font.character_size.height as i32;
        let baseline_y = self.rect.top_left.y + self.rect.size.height as i32 / 2 + glyph_h / 3;

        for col in 0..self.columns {
            let cell_x = self.rect.top_left.x + (col_w * col as u32) as i32;
            let cell_rect = Rectangle::new(
                Point::new(cell_x, self.rect.top_left.y),
                Size::new(col_w, self.rect.size.height),
            );

            // Pressed/selected cell highlight.
            let highlighted = self.pressed_col == Some(col) || self.selected_col == Some(col);
            let text_color = if highlighted {
                renderer.fill_rect(cell_rect, theme.accent.pressed)?;
                theme.accent.on_base
            } else {
                theme.primary.on_base
            };
            if self.focused_col == Some(col) {
                renderer.stroke_rect(cell_rect, theme.accent.base)?;
            }

            if let Some(text) = self.cells.get(col) {
                renderer.draw_text(
                    text,
                    Point::new(cell_x + CELL_PADDING_X as i32, baseline_y),
                    font,
                    text_color,
                    Alignment::Left,
                )?;
            }

            // Vertical grid line between columns (not before the first).
            if col > 0 {
                let line = Rectangle::new(
                    Point::new(cell_x, self.rect.top_left.y),
                    Size::new(1, self.rect.size.height),
                );
                renderer.fill_rect(line, theme.primary.divider)?;
            }
        }

        // Bottom divider.
        let y = self.rect.top_left.y + self.rect.size.height as i32 - 1;
        let divider = Rectangle::new(
            Point::new(self.rect.top_left.x, y),
            Size::new(self.rect.size.width, 1),
        );
        renderer.fill_rect(divider, theme.primary.divider)?;

        Ok(())
    }
}

/// Vertically scrollable table of borrowed text cells with an optional header.
///
/// Build it with [`Table::new`], set columns/data with [`Table::rows`] or
/// [`Table::row`], optionally add a [`Table::header`], then enable scrolling
/// with the same builders [`Column`] exposes. A tapped body cell emits the
/// [`Table::on_select`] message carrying `(row, col)`.
pub struct Table<'a, C: PixelColor, M: Clone> {
    /// Stable base id for body cells.
    id: Option<WidgetId>,
    /// Optional header cells (drawn above the scrolling body).
    header: Option<&'a [&'a str]>,
    /// Body rows, each a borrowed slice of cell strings.
    body: Vec<&'a [&'a str]>,
    /// Column count; defaults to the widest row / header seen.
    columns: usize,
    /// Shared select callback handed to each body row.
    on_select: Option<Rc<dyn Fn(usize, usize) -> M + 'a>>,
    /// `(row, col)` of the host-selected cell to highlight.
    selected: Option<(usize, usize)>,
    /// Whether to zebra-stripe alternating body rows.
    striped: bool,
    width: Length,
    height: Length,
    // Scroll surface, forwarded onto the inner body column.
    scroll_dir: Option<ScrollDirection>,
    scroll_state: Option<ScrollState>,
    scrollbar: Option<ScrollbarMode>,
    snap: Option<SnapMode>,
    on_scroll: Option<Box<dyn Fn(ScrollMsg) -> M + 'a>>,
    /// Cached arranged rect.
    rect: Rectangle,
    /// The composed scrollable body column; built lazily in `arrange`.
    inner: Option<Column<'a, C, M>>,
    /// Header rect captured in `arrange` for drawing.
    header_rect: Rectangle,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Table<'a, C, M> {
    /// Create a new empty table. Position and size are assigned by the parent
    /// via `arrange`.
    pub fn new() -> Self {
        Self {
            header: None,
            body: Vec::new(),
            columns: 0,
            id: None,
            on_select: None,
            selected: None,
            striped: true,
            width: Length::Fill,
            height: Length::Fill,
            scroll_dir: None,
            scroll_state: None,
            scrollbar: None,
            snap: None,
            on_scroll: None,
            rect: Rectangle::zero(),
            inner: None,
            header_rect: Rectangle::zero(),
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

    /// Set a stable base id so body cells can participate in focus traversal.
    #[must_use]
    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    /// The header row, drawn above (and outside) the scrolling body.
    #[must_use]
    pub fn header(mut self, cells: &'a [&'a str]) -> Self {
        self.columns = self.columns.max(cells.len());
        self.header = Some(cells);
        self
    }

    /// Replace all body rows at once from a borrowed 2-D slice.
    #[must_use]
    pub fn rows(mut self, rows: &'a [&'a [&'a str]]) -> Self {
        self.body.clear();
        for r in rows {
            self.columns = self.columns.max(r.len());
            self.body.push(r);
        }
        self
    }

    /// Append a single body row.
    #[must_use]
    pub fn row(mut self, cells: &'a [&'a str]) -> Self {
        self.columns = self.columns.max(cells.len());
        self.body.push(cells);
        self
    }

    /// Explicit column count. Overrides the auto-derived width;
    /// useful when some rows are short.
    #[must_use]
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    /// Zebra-stripe alternating rows (default `true`).
    #[must_use]
    pub fn striped(mut self, on: bool) -> Self {
        self.striped = on;
        self
    }

    /// `(row, col)` of the cell to render highlighted. Host-driven —
    /// typically the value the last [`Table::on_select`] set.
    #[must_use]
    pub fn selected(mut self, row: usize, col: usize) -> Self {
        self.selected = Some((row, col));
        self
    }

    /// Callback invoked with a tapped body cell's `(row, col)`.
    /// Without it the body cells are inert.
    #[must_use]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, usize) -> M + 'a,
    {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Make this table scrollable on `dir`. Tables scroll
    /// vertically, so [`ScrollDirection::Vertical`] is the usual choice.
    #[must_use]
    pub fn scrollable(mut self, dir: ScrollDirection) -> Self {
        self.scroll_dir = Some(dir);
        self
    }

    /// Supply the host-owned [`ScrollState`] read this frame.
    /// Implies scrolling (vertical by default).
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        self.scroll_state = Some(*state);
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// When the scrollbar is drawn.
    #[must_use]
    pub fn scrollbar(mut self, mode: ScrollbarMode) -> Self {
        self.scrollbar = Some(mode);
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// Snapping mode.
    #[must_use]
    pub fn snap(mut self, mode: SnapMode) -> Self {
        self.snap = Some(mode);
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// Callback mapping a [`ScrollMsg`] to the host message.
    #[must_use]
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(ScrollMsg) -> M + 'a,
    {
        self.on_scroll = Some(Box::new(f));
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// Build the scrollable body column from the borrowed rows, applying
    /// striping/select settings.
    fn build_body(&mut self) -> Column<'a, C, M> {
        let mut col = Column::new()
            .width(self.width)
            .height(Length::Fill)
            .spacing(0);

        if let Some(dir) = self.scroll_dir {
            col = col.scrollable(dir);
            if let Some(state) = self.scroll_state.as_ref() {
                col = col.scroll_state(state);
            }
            if let Some(bar) = self.scrollbar {
                col = col.scrollbar(bar);
            }
            if let Some(snap) = self.snap {
                col = col.snap(snap);
            }
            if let Some(on_scroll) = self.on_scroll.take() {
                col = col.on_scroll(move |sm| on_scroll(sm));
            }
        }

        let columns = self.columns.max(1);
        let striped = self.striped;
        let selected = self.selected;
        let on_select = self.on_select.clone();
        for (i, cells) in self.body.iter().copied().enumerate() {
            let mut row = TableRow::new(i, cells, columns);
            row.base_id = self.row_base_id(i);
            row.alternate = striped && (i % 2 == 1);
            row.selected_col = match selected {
                Some((r, c)) if r == i => Some(c),
                _ => None,
            };
            row.on_select = on_select.clone();
            col = col.push(row);
        }
        col
    }

    fn row_base_id(&self, row: usize) -> Option<WidgetId> {
        let columns = self.columns.max(1) as u64;
        self.id.map(|base| {
            WidgetId::new(
                base.raw()
                    .wrapping_add(1)
                    .wrapping_add(row as u64 * columns),
            )
        })
    }

    fn cell_id(&self, row: usize, col: usize) -> Option<WidgetId> {
        self.row_base_id(row)
            .map(|base| WidgetId::new(base.raw().wrapping_add(col as u64)))
    }

    fn coords_for(&self, target: WidgetId) -> Option<(usize, usize)> {
        let columns = self.columns.max(1);
        for row in 0..self.body.len() {
            for col in 0..columns {
                if self.cell_id(row, col) == Some(target) {
                    return Some((row, col));
                }
            }
        }
        None
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Table<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Table<'a, C, M> {
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
        // Reserve a header strip at the top; the body fills the remainder.
        let header_h = if self.header.is_some() {
            TABLE_ROW_HEIGHT.min(rect.size.height)
        } else {
            0
        };
        self.header_rect = Rectangle::new(rect.top_left, Size::new(rect.size.width, header_h));
        let body_rect = Rectangle::new(
            Point::new(rect.top_left.x, rect.top_left.y + header_h as i32),
            Size::new(rect.size.width, rect.size.height.saturating_sub(header_h)),
        );

        let mut body = self.build_body();
        body.arrange(body_rect);
        self.inner = Some(body);
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.inner
            .as_mut()
            .and_then(|body| body.handle_touch(point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(body) = self.inner.as_mut() {
            body.mark_pressed(point);
        }
    }

    fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        if let Some(body) = self.inner.as_ref() {
            body.collect_focusable(out);
        }
    }

    fn sync_focus(&mut self, focused: Option<WidgetId>) {
        if let Some(body) = self.inner.as_mut() {
            body.sync_focus(focused);
        }
    }

    fn route_action(&mut self, target: WidgetId, action: UiAction) -> Option<M> {
        self.inner
            .as_mut()
            .and_then(|body| body.route_action(target, action))
    }

    fn navigate_focus(&self, target: WidgetId, action: UiAction) -> Option<WidgetId> {
        let (row, col) = self.coords_for(target)?;
        let columns = self.columns.max(1);
        let (next_row, next_col) = match action {
            UiAction::NavigateLeft => (row, col.saturating_sub(1)),
            UiAction::NavigateRight => (row, (col + 1).min(columns.saturating_sub(1))),
            UiAction::NavigateUp => (row.saturating_sub(1), col),
            UiAction::NavigateDown => ((row + 1).min(self.body.len().saturating_sub(1)), col),
            _ => return None,
        };
        self.cell_id(next_row, next_col)
    }

    fn focus_rect(&self, target: WidgetId) -> Option<Rectangle> {
        self.inner.as_ref().and_then(|body| body.focus_rect(target))
    }

    fn focus_at(&self, point: Point) -> Option<WidgetId> {
        self.inner.as_ref().and_then(|body| body.focus_at(point))
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        // Body first (scrolls under the header strip).
        if let Some(body) = self.inner.as_ref() {
            body.draw(renderer, theme)?;
        }

        // Header on top, so the scrolling body never overlaps it.
        if let Some(cells) = self.header {
            let r = self.header_rect;
            renderer.fill_rect(r, theme.accent.base)?;
            let font = theme.default_font();
            let cols = self.columns.max(1) as u32;
            let col_w = r.size.width / cols;
            let glyph_h = font.character_size.height as i32;
            let baseline_y = r.top_left.y + r.size.height as i32 / 2 + glyph_h / 3;
            for (col, text) in cells.iter().enumerate() {
                let cell_x = r.top_left.x + (col_w * col as u32) as i32;
                renderer.draw_text(
                    text,
                    Point::new(cell_x + CELL_PADDING_X as i32, baseline_y),
                    font,
                    theme.accent.on_base,
                    Alignment::Left,
                )?;
                if col > 0 {
                    let line = Rectangle::new(
                        Point::new(cell_x, r.top_left.y),
                        Size::new(1, r.size.height),
                    );
                    renderer.fill_rect(line, theme.accent.border)?;
                }
            }
            // Bottom edge of the header.
            let y = r.top_left.y + r.size.height as i32 - 1;
            let edge = Rectangle::new(Point::new(r.top_left.x, y), Size::new(r.size.width, 1));
            renderer.fill_rect(edge, theme.accent.border)?;
        }

        Ok(())
    }
}
