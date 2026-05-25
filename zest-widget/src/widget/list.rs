//! Styled, vertically scrollable list of selectable rows.
//!
//! `List` wraps a scrollable [`Column`]: it collects rows added through its
//! builders, lays them out with consistent padding and optional dividers,
//! gives each row a pressed highlight, and reports a tapped row index through
//! [`List::on_select`]. It exposes the same scroll surface as `Column` —
//! [`List::scrollable`], [`List::scroll_state`], [`List::scrollbar`],
//! [`List::snap`], and [`List::on_scroll`].
//!
//! ## Composition
//!
//! Internally the rows are wrapped in [`ListRow`] widgets (padding + pressed
//! highlight + select callback) and pushed into a [`Column`] configured with
//! the requested scroll settings. Every `Widget` method delegates to that
//! inner column, so layout/touch/draw/scroll behavior matches `Column`.
//!
//! ## Row colors
//!
//! Rows are drawn with `theme.primary` colors: the resting background is
//! `primary.base`, the pressed highlight is `theme.accent.base`, and an
//! optional per-row divider uses `primary.divider`. Text leans on the
//! theme's default font and `primary.on_base`.

use super::{Widget, column::Column};
use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{
    Constraints, Length, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState,
    ScrollbarMode, SnapMode, TouchPhase,
};
use zest_theme::Theme;

/// Default height (px) of a list row.
pub const ROW_HEIGHT: u32 = 44;
/// Horizontal padding (px) applied inside each row.
pub const ROW_PADDING_X: u32 = 12;
/// Gap (px) between a row's leading slot, label, and trailing slot.
pub const ROW_GAP: u32 = 8;

/// A single styled, selectable list row.
///
/// Holds optional leading/trailing text slots (for icon glyphs or accessory
/// text) plus a main label. Draws a pressed highlight, optional bottom
/// divider, and emits its owning [`List`]'s `on_select` message — carrying
/// this row's index — on tap. The press/`mark_pressed` semantics mirror
/// [`Button`](crate::Button) so drag-off-to-cancel and tap-vs-scroll work.
pub struct ListRow<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    index: usize,
    leading: Option<String>,
    label: String,
    trailing: Option<String>,
    /// Shared select callback owned by the parent `List`.
    on_select: Option<Rc<dyn Fn(usize) -> M + 'a>>,
    /// Whether to draw a bottom divider under this row.
    divider: bool,
    /// Whether this row should render as selected (host-driven highlight).
    selected: bool,
    pressed: bool,
    width: Length,
    height: Length,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> ListRow<'a, C, M> {
    fn new(index: usize, label: impl Into<String>) -> Self {
        Self {
            rect: Rectangle::zero(),
            index,
            leading: None,
            label: label.into(),
            trailing: None,
            on_select: None,
            divider: false,
            selected: false,
            pressed: false,
            width: Length::Fill,
            height: Length::Fixed(ROW_HEIGHT),
            _color: PhantomData,
        }
    }

    /// True iff a select callback is bound.
    fn is_enabled(&self) -> bool {
        self.on_select.is_some()
    }

    fn hit_test(&self, point: Point) -> bool {
        let tl = self.rect.top_left;
        let br = tl + Point::new(self.rect.size.width as i32, self.rect.size.height as i32);
        point.x >= tl.x && point.x < br.x && point.y >= tl.y && point.y < br.y
    }
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for ListRow<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self.height.resolve(ROW_HEIGHT, constraints.max.height);
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
        if !self.is_enabled() || !self.hit_test(point) {
            if matches!(phase, TouchPhase::Up | TouchPhase::Moved) {
                self.pressed = false;
            }
            return None;
        }
        match phase {
            TouchPhase::Down => {
                self.pressed = true;
                None
            }
            TouchPhase::Up => {
                if self.pressed {
                    self.pressed = false;
                    self.on_select.as_ref().map(|cb| cb(self.index))
                } else {
                    None
                }
            }
            TouchPhase::Moved => None,
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.is_enabled() && self.hit_test(point) {
            self.pressed = true;
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let font = theme.default_font();
        // Background: pressed/selected highlight else the panel base.
        if self.pressed {
            renderer.fill_rect(self.rect, theme.accent.pressed)?;
        } else if self.selected {
            renderer.fill_rect(self.rect, theme.accent.base)?;
        } else {
            renderer.fill_rect(self.rect, theme.primary.base)?;
        }

        let text_color = if self.pressed || self.selected {
            theme.accent.on_base
        } else {
            theme.primary.on_base
        };

        let glyph_h = font.character_size.height as i32;
        let baseline_y = self.rect.top_left.y + self.rect.size.height as i32 / 2 + glyph_h / 3;
        let left_x = self.rect.top_left.x + ROW_PADDING_X as i32;
        let right_x = self.rect.top_left.x + self.rect.size.width as i32 - ROW_PADDING_X as i32;

        // Leading slot (e.g. an icon glyph).
        let mut label_x = left_x;
        if let Some(leading) = &self.leading {
            renderer.draw_text(
                leading,
                Point::new(left_x, baseline_y),
                font,
                text_color,
                Alignment::Left,
            )?;
            let advance = font.character_size.width as i32 * leading.chars().count() as i32;
            label_x = left_x + advance + ROW_GAP as i32;
        }

        // Main label.
        renderer.draw_text(
            &self.label,
            Point::new(label_x, baseline_y),
            font,
            text_color,
            Alignment::Left,
        )?;

        // Trailing slot, right-aligned.
        if let Some(trailing) = &self.trailing {
            renderer.draw_text(
                trailing,
                Point::new(right_x, baseline_y),
                font,
                text_color,
                Alignment::Right,
            )?;
        }

        // Bottom divider.
        if self.divider {
            let y = self.rect.top_left.y + self.rect.size.height as i32 - 1;
            let divider = Rectangle::new(
                Point::new(self.rect.top_left.x, y),
                Size::new(self.rect.size.width, 1),
            );
            renderer.fill_rect(divider, theme.primary.divider)?;
        }

        Ok(())
    }
}

/// Styled, scrollable vertical list of selectable rows.
///
/// Build it with [`List::new`], add rows via [`List::item`],
/// [`List::item_with`], or [`List::push`], then enable scrolling with the
/// same builders a [`Column`] exposes. A tapped row emits the
/// [`List::on_select`] message carrying its zero-based index.
pub struct List<'a, C: PixelColor, M: Clone> {
    /// Pending rows, materialized into the inner column at layout time.
    rows: Vec<ListRow<'a, C, M>>,
    /// Shared select callback handed to each row.
    on_select: Option<Rc<dyn Fn(usize) -> M + 'a>>,
    /// Index of a row to render highlighted (host-driven selection).
    selected: Option<usize>,
    /// Whether to draw a divider under every row.
    dividers: bool,
    spacing: u32,
    width: Length,
    height: Length,
    // Scroll surface, forwarded verbatim onto the inner column.
    scroll_dir: Option<ScrollDirection>,
    scroll_state: Option<ScrollState>,
    scrollbar: Option<ScrollbarMode>,
    snap: Option<SnapMode>,
    on_scroll: Option<Box<dyn Fn(ScrollMsg) -> M + 'a>>,
    /// The composed scrollable column; built lazily in `arrange`.
    inner: Option<Column<'a, C, M>>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> List<'a, C, M> {
    /// Create a new empty list. Position and size are assigned by the parent
    /// via `arrange`. Defaults to filling its container with 0 px spacing
    /// between rows.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            on_select: None,
            selected: None,
            dividers: false,
            spacing: 0,
            width: Length::Fill,
            height: Length::Fill,
            scroll_dir: None,
            scroll_state: None,
            scrollbar: None,
            snap: None,
            on_scroll: None,
            inner: None,
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

    /// Gap (px) between rows. Defaults to 0 (rows abut, suited to
    /// dividers).
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Draw a thin divider under every row.
    #[must_use]
    pub fn dividers(mut self, on: bool) -> Self {
        self.dividers = on;
        self
    }

    /// Index of the row to render with the selected highlight.
    /// Host-driven — typically the value the last [`List::on_select`] set.
    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// Callback invoked with a tapped row's index. Without it the
    /// rows are inert (they still draw, but emit no message).
    #[must_use]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> M + 'a,
    {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Append a row showing a single `label`.
    #[must_use]
    pub fn item(mut self, label: impl Into<String>) -> Self {
        let index = self.rows.len();
        self.rows.push(ListRow::new(index, label));
        self
    }

    /// Append a row with an optional leading slot (e.g. an icon
    /// glyph), a main `label`, and an optional trailing slot (e.g. an
    /// accessory glyph or value text). Pass `""`/`None` for slots you don't
    /// need.
    #[must_use]
    pub fn item_with(
        mut self,
        leading: Option<impl Into<String>>,
        label: impl Into<String>,
        trailing: Option<impl Into<String>>,
    ) -> Self {
        let index = self.rows.len();
        let mut row = ListRow::new(index, label);
        row.leading = leading.map(Into::into);
        row.trailing = trailing.map(Into::into);
        self.rows.push(row);
        self
    }

    /// Append a pre-built [`ListRow`]. The row's index is reassigned
    /// to its position in the list, and the list-wide divider/select settings
    /// are applied at layout time.
    #[must_use]
    pub fn push(mut self, mut row: ListRow<'a, C, M>) -> Self {
        row.index = self.rows.len();
        self.rows.push(row);
        self
    }

    /// Make this list scrollable on `dir`. Mirrors
    /// [`Column::scrollable`]. Lists are vertical, so
    /// [`ScrollDirection::Vertical`] is the usual choice.
    #[must_use]
    pub fn scrollable(mut self, dir: ScrollDirection) -> Self {
        self.scroll_dir = Some(dir);
        self
    }

    /// Supply the host-owned [`ScrollState`] read this frame.
    /// Implies scrolling (vertical by default). Mirrors
    /// [`Column::scroll_state`].
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        self.scroll_state = Some(*state);
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// When the scrollbar is drawn. Mirrors [`Column::scrollbar`].
    #[must_use]
    pub fn scrollbar(mut self, mode: ScrollbarMode) -> Self {
        self.scrollbar = Some(mode);
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// Snapping mode. Mirrors [`Column::snap`].
    #[must_use]
    pub fn snap(mut self, mode: SnapMode) -> Self {
        self.snap = Some(mode);
        if self.scroll_dir.is_none() {
            self.scroll_dir = Some(ScrollDirection::Vertical);
        }
        self
    }

    /// Callback mapping a [`ScrollMsg`] to the host message. Mirrors
    /// [`Column::on_scroll`].
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

    /// Materialize the pending rows into the internal scrollable column,
    /// applying the list-wide divider/select/selected settings. Consumes the
    /// row vector, so this is called once per `arrange`.
    fn build_inner(&mut self) -> Column<'a, C, M> {
        let mut col = Column::new()
            .width(self.width)
            .height(self.height)
            .spacing(self.spacing);

        // Wire the scroll surface only when scrolling was requested.
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

        let dividers = self.dividers;
        let selected = self.selected;
        let on_select = self.on_select.clone();
        for mut row in core::mem::take(&mut self.rows) {
            row.divider = dividers;
            row.selected = Some(row.index) == selected;
            row.on_select = on_select.clone();
            col = col.push(row);
        }
        col
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for List<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for List<'a, C, M> {
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
        // Rebuild the composed column each frame (rows/scroll state are fresh)
        // and arrange it into our slot.
        let mut col = self.build_inner();
        col.arrange(rect);
        self.inner = Some(col);
    }

    fn rect(&self) -> Rectangle {
        self.inner.as_ref().map_or(Rectangle::zero(), Widget::rect)
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        self.inner
            .as_mut()
            .and_then(|col| col.handle_touch(point, phase))
    }

    fn mark_pressed(&mut self, point: Point) {
        if let Some(col) = self.inner.as_mut() {
            col.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        if let Some(col) = self.inner.as_ref() {
            col.draw(renderer, theme)?;
        }
        Ok(())
    }
}
