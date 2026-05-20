//! Month-view calendar with prev/next nav, selected-day highlight,
//! and per-day event-color dots.
//!
//! Date math lives in the caller — this widget asks for
//! `days_in_month` and `first_day_of_week` (0 = Sun … 6 = Sat) plus a
//! flat list of `(day, color)` event markers. That keeps the widget
//! independent of any date library (`chrono`, `time`, hand-rolled) and
//! the no_std story clean.
//!
//! Interaction emits user messages via builder callbacks:
//! - tap on a day cell → `on_select(day)` is invoked
//! - tap on the left header arrow → `on_prev` message fires
//! - tap on the right header arrow → `on_next` message fires
//!
//! Layout (assumes the widget gets roughly 320x208 on a 320x240
//! panel with a tab bar): 24-px header strip, 16-px day-of-week row,
//! six week rows filling the rest.

use super::Widget;
use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::{ButtonCatalog, ButtonClass, Status, Theme};

const HEADER_H: u32 = 24;
const DOW_H: u32 = 16;
const NAV_W: u32 = 36;
const WEEK_ROWS: u32 = 6;
const COLS: u32 = 7;

const DAY_HOUR_H: u32 = 18;
const DAY_LABEL_W: u32 = 40;

/// Which view mode the calendar should render.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum CalendarMode {
    /// Month grid (default).
    #[default]
    Month,
    /// 24-row day schedule. Caller usually wraps the calendar in a
    /// `Scrollable` since the day view's intrinsic height (≈ 24 +
    /// 24×18 ≈ 456 px) is taller than typical embedded viewports.
    Day,
}

/// One event marker on the calendar — a day-of-month plus an
/// indicator color. In Day-view mode, the optional `time` (hour 0–23,
/// minute 0–59) places the event in its hour row; events with `time =
/// None` are skipped in Day view.
#[derive(Clone, Debug)]
pub struct CalendarEvent<C: PixelColor> {
    /// Day-of-month, 1-based.
    pub day: u32,
    /// Color of the dot drawn inside the day's cell.
    pub color: C,
    /// Optional `(hour, minute)` for Day-view placement.
    pub time: Option<(u8, u8)>,
    /// Label text rendered inside the event bar in Day view. Unused
    /// in Month view (which only paints dots).
    pub label: String,
}

/// Month-view calendar widget.
pub struct Calendar<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    year: i32,
    month: u32,
    month_name: String,
    days_in_month: u32,
    first_dow: u8,
    selected_day: Option<u32>,
    today: Option<u32>,
    events: Vec<CalendarEvent<C>>,
    on_select: Option<Box<dyn Fn(u32) -> M + 'a>>,
    on_prev: Option<M>,
    on_next: Option<M>,
    width: Length,
    height: Length,
    pressed_day: Option<u32>,
    pressed_nav: Option<NavZone>,
    mode: CalendarMode,
    day_label: String,
    _phantom: PhantomData<C>,
}

impl<'a, C: PixelColor, M: Clone> Calendar<'a, C, M> {
    /// New calendar for the given month. `month_name` is rendered in
    /// the header — supply something like `"March 2026"`.
    pub fn new(year: i32, month: u32, month_name: impl Into<String>) -> Self {
        Self {
            rect: Rectangle::zero(),
            year,
            month,
            month_name: month_name.into(),
            days_in_month: 30,
            first_dow: 0,
            selected_day: None,
            today: None,
            events: Vec::new(),
            on_select: None,
            on_prev: None,
            on_next: None,
            width: Length::Fill,
            height: Length::Fill,
            pressed_day: None,
            pressed_nav: None,
            mode: CalendarMode::Month,
            day_label: String::new(),
            _phantom: PhantomData,
        }
    }

    /// Builder: rendering mode (Month grid or Day schedule).
    #[must_use]
    pub fn mode(mut self, mode: CalendarMode) -> Self {
        self.mode = mode;
        self
    }

    /// Builder: text label rendered in the header in Day-view mode
    /// (e.g. `"Mon, Mar 14"`). Ignored in Month mode.
    #[must_use]
    pub fn day_label(mut self, label: impl Into<String>) -> Self {
        self.day_label = label.into();
        self
    }

    /// Builder: number of days in this month (caller's date math).
    #[must_use]
    pub fn days_in_month(mut self, n: u32) -> Self {
        self.days_in_month = n;
        self
    }

    /// Builder: day-of-week of the 1st of this month. 0=Sun … 6=Sat.
    #[must_use]
    pub fn first_day_of_week(mut self, d: u8) -> Self {
        self.first_dow = d.min(6);
        self
    }

    /// Builder: highlight a particular day as the user's current selection.
    #[must_use]
    pub fn selected(mut self, day: u32) -> Self {
        self.selected_day = Some(day);
        self
    }

    /// Builder: mark a day as "today" (subtle ring around the cell).
    /// Optional; pass when you want today to stand out without selecting it.
    #[must_use]
    pub fn today(mut self, day: u32) -> Self {
        self.today = Some(day);
        self
    }

    /// Builder: replace the event list outright.
    #[must_use]
    pub fn events(mut self, events: impl IntoIterator<Item = CalendarEvent<C>>) -> Self {
        self.events = events.into_iter().collect();
        self
    }

    /// Builder: add one event marker (no time, empty label).
    #[must_use]
    pub fn event(mut self, day: u32, color: C) -> Self {
        self.events.push(CalendarEvent {
            day,
            color,
            time: None,
            label: String::new(),
        });
        self
    }

    /// Builder: callback fired when a day cell is tapped.
    #[must_use]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(u32) -> M + 'a,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Builder: message fired when the previous-month arrow is tapped.
    #[must_use]
    pub fn on_prev(mut self, msg: M) -> Self {
        self.on_prev = Some(msg);
        self
    }

    /// Builder: message fired when the next-month arrow is tapped.
    #[must_use]
    pub fn on_next(mut self, msg: M) -> Self {
        self.on_next = Some(msg);
        self
    }

    /// Builder: width sizing intent.
    #[must_use]
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// Builder: height sizing intent.
    #[must_use]
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    /// Year currently displayed.
    #[must_use]
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Month currently displayed (1–12).
    #[must_use]
    pub fn month(&self) -> u32 {
        self.month
    }

    fn header_rect(&self) -> Rectangle {
        Rectangle::new(self.rect.top_left, Size::new(self.rect.size.width, HEADER_H))
    }

    fn prev_rect(&self) -> Rectangle {
        Rectangle::new(self.rect.top_left, Size::new(NAV_W, HEADER_H))
    }

    fn next_rect(&self) -> Rectangle {
        let w = self.rect.size.width;
        Rectangle::new(
            self.rect.top_left + Point::new(w.saturating_sub(NAV_W) as i32, 0),
            Size::new(NAV_W, HEADER_H),
        )
    }

    fn dow_rect(&self) -> Rectangle {
        Rectangle::new(
            self.rect.top_left + Point::new(0, HEADER_H as i32),
            Size::new(self.rect.size.width, DOW_H),
        )
    }

    fn grid_rect(&self) -> Rectangle {
        let top = HEADER_H + DOW_H;
        Rectangle::new(
            self.rect.top_left + Point::new(0, top as i32),
            Size::new(
                self.rect.size.width,
                self.rect.size.height.saturating_sub(top),
            ),
        )
    }

    fn cell_size(&self) -> Size {
        let grid = self.grid_rect();
        Size::new(grid.size.width / COLS, grid.size.height / WEEK_ROWS)
    }

    fn cell_rect(&self, row: u32, col: u32) -> Rectangle {
        let grid = self.grid_rect();
        let cell = self.cell_size();
        Rectangle::new(
            grid.top_left + Point::new((col * cell.width) as i32, (row * cell.height) as i32),
            cell,
        )
    }

    fn day_at(&self, idx: u32) -> Option<u32> {
        let first = self.first_dow as u32;
        if idx < first {
            return None;
        }
        let day = idx - first + 1;
        if day > self.days_in_month {
            None
        } else {
            Some(day)
        }
    }

    fn hit_test_day(&self, point: Point) -> Option<u32> {
        let grid = self.grid_rect();
        if !rect_contains(grid, point) {
            return None;
        }
        let cell = self.cell_size();
        if cell.width == 0 || cell.height == 0 {
            return None;
        }
        let col = ((point.x - grid.top_left.x) as u32 / cell.width).min(COLS - 1);
        let row = ((point.y - grid.top_left.y) as u32 / cell.height).min(WEEK_ROWS - 1);
        self.day_at(row * COLS + col)
    }

    fn hit_test_nav(&self, point: Point) -> Option<NavZone> {
        if rect_contains(self.prev_rect(), point) {
            Some(NavZone::Prev)
        } else if rect_contains(self.next_rect(), point) {
            Some(NavZone::Next)
        } else {
            None
        }
    }

    fn day_intrinsic_height(&self) -> u32 {
        HEADER_H + 24 * DAY_HOUR_H
    }

    fn day_hour_rect(&self, hour: u32) -> Rectangle {
        Rectangle::new(
            self.rect.top_left + Point::new(0, (HEADER_H + hour * DAY_HOUR_H) as i32),
            Size::new(self.rect.size.width, DAY_HOUR_H),
        )
    }

    fn hit_test_hour(&self, point: Point) -> Option<u32> {
        if point.y < self.rect.top_left.y + HEADER_H as i32 {
            return None;
        }
        if point.x < self.rect.top_left.x
            || point.x >= self.rect.top_left.x + self.rect.size.width as i32
        {
            return None;
        }
        let offset = (point.y - self.rect.top_left.y) as u32;
        if offset < HEADER_H {
            return None;
        }
        let hour = (offset - HEADER_H) / DAY_HOUR_H;
        if hour < 24 { Some(hour) } else { None }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NavZone {
    Prev,
    Next,
}

fn rect_contains(rect: Rectangle, p: Point) -> bool {
    let top_left = rect.top_left;
    let bottom_right =
        top_left + Point::new(rect.size.width as i32, rect.size.height as i32);
    p.x >= top_left.x && p.x < bottom_right.x && p.y >= top_left.y && p.y < bottom_right.y
}

impl<'a, C: PixelColor, M: Clone> Widget<C, M> for Calendar<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let intrinsic_h = match self.mode {
            CalendarMode::Month => HEADER_H + DOW_H + WEEK_ROWS * 24,
            CalendarMode::Day => self.day_intrinsic_height(),
        };
        let w = self.width.resolve(constraints.max.width, constraints.max.width);
        let h = match self.mode {
            CalendarMode::Day => intrinsic_h,
            CalendarMode::Month => self.height.resolve(intrinsic_h, constraints.max.height),
        };
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        match self.mode {
            CalendarMode::Day => (self.width, Length::Shrink),
            CalendarMode::Month => (self.width, self.height),
        }
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        match phase {
            TouchPhase::Down => {
                self.pressed_nav = self.hit_test_nav(point);
                self.pressed_day = if self.pressed_nav.is_some() {
                    None
                } else {
                    match self.mode {
                        CalendarMode::Month => self.hit_test_day(point),
                        CalendarMode::Day => self.hit_test_hour(point),
                    }
                };
                None
            }
            TouchPhase::Moved => {
                let now = match self.mode {
                    CalendarMode::Month => self.hit_test_day(point),
                    CalendarMode::Day => self.hit_test_hour(point),
                };
                if self.pressed_day.is_some() && now != self.pressed_day {
                    self.pressed_day = None;
                }
                if self.pressed_nav.is_some() && self.hit_test_nav(point) != self.pressed_nav {
                    self.pressed_nav = None;
                }
                None
            }
            TouchPhase::Up => {
                let nav_now = self.hit_test_nav(point);
                let nav_pressed = self.pressed_nav.take();
                if let (Some(z), Some(p)) = (nav_now, nav_pressed) {
                    if z == p {
                        return match z {
                            NavZone::Prev => self.on_prev.clone(),
                            NavZone::Next => self.on_next.clone(),
                        };
                    }
                }
                let cell = match self.mode {
                    CalendarMode::Month => self.hit_test_day(point),
                    CalendarMode::Day => self.hit_test_hour(point),
                };
                let pressed = self.pressed_day.take();
                if let (Some(d), Some(p), Some(cb)) = (cell, pressed, self.on_select.as_ref()) {
                    if d == p {
                        return Some(cb(d));
                    }
                }
                None
            }
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.pressed_nav.is_none() {
            self.pressed_nav = self.hit_test_nav(point);
        }
        if self.pressed_nav.is_none() && self.pressed_day.is_none() {
            self.pressed_day = match self.mode {
                CalendarMode::Month => self.hit_test_day(point),
                CalendarMode::Day => self.hit_test_hour(point),
            };
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        renderer.fill_rect(self.rect, theme.background.base)?;
        self.draw_header(renderer, theme)?;
        match self.mode {
            CalendarMode::Month => self.draw_month_grid(renderer, theme)?,
            CalendarMode::Day => self.draw_day_schedule(renderer, theme)?,
        }
        Ok(())
    }
}

impl<'a, C: PixelColor, M: Clone> Calendar<'a, C, M> {
    fn draw_header<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let header = self.header_rect();
        renderer.fill_rect(header, theme.primary.base)?;

        let prev_rect = self.prev_rect();
        let next_rect = self.next_rect();
        let prev_status = if self.pressed_nav == Some(NavZone::Prev) {
            Status::Pressed
        } else if self.on_prev.is_some() {
            Status::Active
        } else {
            Status::Disabled
        };
        let next_status = if self.pressed_nav == Some(NavZone::Next) {
            Status::Pressed
        } else if self.on_next.is_some() {
            Status::Active
        } else {
            Status::Disabled
        };
        let prev = theme.button(ButtonClass::Standard, prev_status);
        let next = theme.button(ButtonClass::Standard, next_status);
        if let Some(bg) = prev.background {
            renderer.fill_rect(prev_rect, bg)?;
        }
        if let Some(border) = prev.border {
            renderer.stroke_rect(prev_rect, border)?;
        }
        if let Some(bg) = next.background {
            renderer.fill_rect(next_rect, bg)?;
        }
        if let Some(border) = next.border {
            renderer.stroke_rect(next_rect, border)?;
        }
        let body = theme.typography.body;
        let baseline_y = header.top_left.y + (header.size.height / 2) as i32
            + (body.character_size.height / 3) as i32;
        renderer.draw_text(
            "<",
            Point::new(prev_rect.top_left.x + (prev_rect.size.width / 2) as i32, baseline_y),
            body,
            prev.text,
            Alignment::Center,
        )?;
        renderer.draw_text(
            ">",
            Point::new(next_rect.top_left.x + (next_rect.size.width / 2) as i32, baseline_y),
            body,
            next.text,
            Alignment::Center,
        )?;
        let label = match self.mode {
            CalendarMode::Month => self.month_name.as_str(),
            CalendarMode::Day => self.day_label.as_str(),
        };
        renderer.draw_text(
            label,
            Point::new(
                header.top_left.x + (header.size.width / 2) as i32,
                baseline_y,
            ),
            body,
            theme.primary.on_base,
            Alignment::Center,
        )?;
        Ok(())
    }

    fn draw_month_grid<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let dow = self.dow_rect();
        let cell_w = dow.size.width / COLS;
        let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for (i, name) in names.iter().enumerate() {
            let x = dow.top_left.x + (i as i32) * cell_w as i32 + (cell_w / 2) as i32;
            let y = dow.top_left.y + (dow.size.height / 2) as i32
                + (theme.typography.caption.character_size.height / 3) as i32;
            renderer.draw_text(
                name,
                Point::new(x, y),
                theme.typography.caption,
                theme.background.divider,
                Alignment::Center,
            )?;
        }

        for idx in 0..(COLS * WEEK_ROWS) {
            let row = idx / COLS;
            let col = idx % COLS;
            let cell = self.cell_rect(row, col);
            let day = match self.day_at(idx) {
                Some(d) => d,
                None => continue,
            };

            let is_selected = Some(day) == self.selected_day;
            let is_today = Some(day) == self.today;
            let is_pressed = Some(day) == self.pressed_day;

            if is_selected {
                let acc = theme.button(ButtonClass::Suggested, Status::Active);
                if let Some(bg) = acc.background {
                    renderer.fill_rect(cell, bg)?;
                }
            } else if is_pressed {
                let std = theme.button(ButtonClass::Standard, Status::Pressed);
                if let Some(bg) = std.background {
                    renderer.fill_rect(cell, bg)?;
                }
            }

            if is_today && !is_selected {
                renderer.stroke_rect(cell, theme.accent.base)?;
            }

            let text_color = if is_selected {
                theme.button(ButtonClass::Suggested, Status::Active).text
            } else {
                theme.background.on_base
            };
            let label = format!("{day}");
            renderer.draw_text(
                &label,
                Point::new(
                    cell.top_left.x + (cell.size.width / 2) as i32,
                    cell.top_left.y + (cell.size.height / 2) as i32
                        + (theme.typography.body.character_size.height / 3) as i32,
                ),
                theme.typography.body,
                text_color,
                Alignment::Center,
            )?;

            let mut dots = self.events.iter().filter(|e| e.day == day);
            let dot_y = cell.top_left.y + cell.size.height as i32 - 4;
            let mut dot_x = cell.top_left.x + (cell.size.width / 2) as i32 - 6;
            for _ in 0..3 {
                let Some(ev) = dots.next() else { break };
                renderer.fill_circle(Point::new(dot_x, dot_y), 1, ev.color)?;
                dot_x += 5;
            }
        }
        Ok(())
    }

    fn draw_day_schedule<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let body = theme.typography.body;
        let baseline_off = (body.character_size.height / 3) as i32;
        for hour in 0..24u32 {
            let row = self.day_hour_rect(hour);
            // Bottom divider for each row.
            renderer.fill_rect(
                Rectangle::new(
                    Point::new(row.top_left.x, row.top_left.y + row.size.height as i32 - 1),
                    Size::new(row.size.width, 1),
                ),
                theme.background.divider,
            )?;
            if self.pressed_day == Some(hour) {
                let s = theme.button(ButtonClass::Standard, Status::Pressed);
                if let Some(bg) = s.background {
                    renderer.fill_rect(row, bg)?;
                }
            }
            let label = format!("{hour:02}:00");
            renderer.draw_text(
                &label,
                Point::new(
                    row.top_left.x + 4,
                    row.top_left.y + (row.size.height / 2) as i32 + baseline_off,
                ),
                theme.typography.caption,
                theme.background.divider,
                Alignment::Left,
            )?;
        }

        // Event bars in their hour rows.
        for ev in &self.events {
            let Some((h, m)) = ev.time else { continue };
            if h as u32 >= 24 {
                continue;
            }
            let row = self.day_hour_rect(h as u32);
            let bar_y_offset = ((m as i32) * (DAY_HOUR_H as i32) / 60).clamp(0, DAY_HOUR_H as i32 - 4);
            let bar = Rectangle::new(
                Point::new(
                    row.top_left.x + DAY_LABEL_W as i32,
                    row.top_left.y + bar_y_offset,
                ),
                Size::new(
                    row.size.width.saturating_sub(DAY_LABEL_W + 4),
                    DAY_HOUR_H.saturating_sub(2),
                ),
            );
            renderer.fill_rect(bar, ev.color)?;
            // Label rendered in body font, on top of the colored bar.
            // Pick a contrasting color: just use background.base (theme
            // primary) — for dark themes this is dark text, for light
            // themes this is light text, which usually contrasts with
            // the saturated event color.
            renderer.draw_text(
                &ev.label,
                Point::new(
                    bar.top_left.x + 4,
                    bar.top_left.y + (bar.size.height / 2) as i32 + baseline_off,
                ),
                theme.typography.caption,
                theme.background.base,
                Alignment::Left,
            )?;
        }
        Ok(())
    }
}
