//! Vertical wheel / drum selector — a scrollable list of option labels that
//! settles each release so the chosen option lands centered under a fixed
//! highlight band.
//!
//! A `Roller` is a self-contained scrollable widget (it does not wrap a
//! [`Column`](super::column::Column)) so it can paint the center highlight
//! band, dim the non-centered rows, and report the centered option to the
//! host. It scrolls vertically with [`SnapMode::Center`]: dragging spins the
//! drum 1:1, and on release the spring settles the nearest option to the
//! viewport center.
//!
//! ## Host ownership
//!
//! Like every scrollable widget in `zest`, the cross-frame scroll state lives
//! on the host (widgets are transient — see [`scroll`](zest_core::scroll)).
//! The host owns a [`ScrollState`] and the selected index, passing the state
//! by reference each frame via [`Roller::scroll_state`]. The roller reads it
//! during layout/draw and emits:
//!
//! - [`ScrollMsg`] through [`Roller::on_scroll`] — apply to the owned
//!   [`ScrollState`] in `update()` exactly as for a scrollable
//!   [`Column`](super::column::Column).
//! - the centered option index through [`Roller::on_select`] — fires when the
//!   item under the highlight band changes (during drag/fling/settle), so the
//!   host can store the live selection.
//!
//! Leading and trailing padding equal to half the viewport (minus half a row)
//! is folded into the content extent so the first and last options can reach
//! the center.

use super::{Widget, scroll_core};
use alloc::{boxed::Box, string::String, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{
    mono_font::MonoFont, pixelcolor::PixelColor, prelude::*, primitives::Rectangle,
    text::Alignment as EgAlignment,
};
use zest_core::{
    Constraints, GesturePhase, Length, RenderError, Renderer, ScrollDirection, ScrollMsg,
    ScrollState, TouchPhase,
};
use zest_theme::Theme;

/// Default pixel height of one option row.
const DEFAULT_ITEM_HEIGHT: u32 = 36;
/// Default number of rows shown in the viewport (kept odd so one row is
/// unambiguously centered).
const DEFAULT_VISIBLE: u32 = 5;

/// Vertical wheel / drum selector.
///
/// Renders its options as a vertically scrollable drum with a fixed center
/// highlight band. The host owns the [`ScrollState`] and the selected index;
/// the roller reads the state, draws, and emits messages.
pub struct Roller<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    /// Option labels, in order.
    options: Vec<String>,
    /// Host-owned scroll state, read each frame.
    state: ScrollState,
    /// Height of one option row in pixels.
    item_height: u32,
    /// Number of rows the viewport shows (height = `visible * item_height`).
    visible: u32,
    /// Host's currently selected index (used to colour the centered row when
    /// the drum is at rest and as a fallback before the first scroll).
    selected: usize,
    /// Optional font override (defaults to the theme body font).
    font: Option<&'a MonoFont<'a>>,
    /// Width sizing intent.
    width: Length,
    /// Callback turning a [`ScrollMsg`] into the host message.
    on_scroll: Option<Box<dyn Fn(ScrollMsg) -> M + 'a>>,
    /// Callback fired with the centered index when it changes.
    on_select: Option<Box<dyn Fn(usize) -> M + 'a>>,
    /// Cached content height (rows + leading/trailing centering pads).
    content_h: u32,
    _color: PhantomData<C>,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Roller<'a, C, M> {
    /// Create an empty roller. Supply options via [`Roller::options`] or
    /// [`Roller::option`], and the host state via [`Roller::scroll_state`].
    /// Position and size are assigned by the parent container via `arrange`.
    pub fn new() -> Self {
        Self {
            rect: Rectangle::zero(),
            options: Vec::new(),
            state: ScrollState::new(),
            item_height: DEFAULT_ITEM_HEIGHT,
            visible: DEFAULT_VISIBLE,
            selected: 0,
            font: None,
            width: Length::Fill,
            on_scroll: None,
            on_select: None,
            content_h: 0,
            _color: PhantomData,
        }
    }

    /// Builder: replace all options from a slice of string slices.
    #[must_use]
    pub fn options(mut self, options: &[&str]) -> Self {
        self.options = options.iter().map(|s| String::from(*s)).collect();
        self
    }

    /// Builder: append a single option label.
    #[must_use]
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(label.into());
        self
    }

    /// Builder: supply the host-owned [`ScrollState`] read this frame.
    #[must_use]
    pub fn scroll_state(mut self, state: &ScrollState) -> Self {
        self.state = *state;
        self
    }

    /// Builder: the host's currently selected option index. Used to colour the
    /// centered row at rest before any scroll has happened.
    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Builder: pixel height of one option row (default 36).
    #[must_use]
    pub fn item_height(mut self, height: u32) -> Self {
        self.item_height = height.max(1);
        self
    }

    /// Builder: number of rows shown in the viewport (default 5). Kept odd so
    /// exactly one row sits under the highlight band.
    #[must_use]
    pub fn visible_count(mut self, count: u32) -> Self {
        self.visible = count.max(1);
        self
    }

    /// Builder: override the option font (defaults to `theme.typography.body`).
    #[must_use]
    pub fn font(mut self, font: &'a MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }

    /// Builder: width sizing intent (height is driven by `visible * item_height`).
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Builder: callback mapping a [`ScrollMsg`] to the host message. Apply the
    /// message to the owned [`ScrollState`] in `update()`.
    #[must_use]
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(ScrollMsg) -> M + 'a,
    {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Builder: callback fired with the centered option index whenever it
    /// changes (during drag, fling, or settle).
    #[must_use]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> M + 'a,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Height (px) of the leading/trailing pad that lets the first and last
    /// options reach the center. Equals half the viewport minus half a row.
    fn pad(&self) -> i32 {
        (self.rect.size.height as i32 - self.item_height as i32).max(0) / 2
    }

    /// Total content extent (leading pad + all rows + trailing pad).
    fn content_height(&self) -> u32 {
        let rows = self.item_height.saturating_mul(self.options.len() as u32);
        rows.saturating_add((self.pad() as u32).saturating_mul(2))
    }

    /// Snap-line offsets (one per option) that center each option under the
    /// band. With the leading pad of `pad`, option `i` is centered when the
    /// scroll offset equals `i * item_height` (the pad cancels with the
    /// center-mode `viewport/2 - item/2` term).
    fn snap_lines(&self) -> Vec<i32> {
        (0..self.options.len() as i32)
            .map(|i| i * self.item_height as i32)
            .collect()
    }

    /// The option index currently nearest the center, given the rendered
    /// offset. Saturated to the valid range.
    fn centered_index(&self) -> usize {
        if self.options.is_empty() {
            return 0;
        }
        let off = scroll_core::render_offset(self.state, ScrollDirection::Vertical).y;
        let ih = self.item_height as i32;
        let raw = (off + ih / 2).div_euclid(ih);
        raw.clamp(0, self.options.len() as i32 - 1) as usize
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Default for Roller<'a, C, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Roller<'a, C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let w = self
            .width
            .resolve(constraints.max.width, constraints.max.width);
        let h = self.item_height.saturating_mul(self.visible);
        constraints.clamp(Size::new(w, h))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, Length::Fixed(self.item_height.saturating_mul(self.visible)))
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
        self.content_h = self.content_height();
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        if self.options.is_empty() {
            return None;
        }
        let viewport = self.rect;
        let content = Size::new(self.rect.size.width, self.content_h);
        let lines = self.snap_lines();
        let on_scroll = self.on_scroll.as_deref();
        // The roller has no interactive children; instead the `forward`
        // closure interprets a tap (an `Up` that never crossed the scroll
        // threshold) as "select the option the finger landed on" and emits the
        // `on_select` message for that row. A drag never reaches this branch —
        // route_touch only forwards `Up` while still `Pressing` — so 1:1
        // panning and fling are untouched.
        let off = scroll_core::render_offset(self.state, ScrollDirection::Vertical).y;
        let band_top = viewport.top_left.y + self.pad();
        let ih = self.item_height as i32;
        let on_select = self.on_select.as_deref();
        let count = self.options.len();
        scroll_core::route_touch(
            self.state,
            ScrollDirection::Vertical,
            viewport,
            content,
            point,
            phase,
            &lines,
            on_scroll,
            |p, ph| {
                if ph != TouchPhase::Up {
                    return None;
                }
                // Convert the tap's y into an option index in content space.
                let content_y = p.y - band_top + off;
                if content_y < 0 {
                    return None;
                }
                let idx = (content_y / ih) as usize;
                if idx >= count {
                    return None;
                }
                on_select.map(|f| f(idx))
            },
        )
    }

    fn mark_pressed(&mut self, _point: Point) {}

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let font = self.font.unwrap_or(theme.typography.body);
        let viewport = self.rect;
        let off = scroll_core::render_offset(self.state, ScrollDirection::Vertical).y;
        let ih = self.item_height as i32;
        let glyph_h = font.character_size.height as i32;
        let center_x = viewport.top_left.x + viewport.size.width as i32 / 2;
        let band_top = viewport.top_left.y + self.pad();
        let centered = self.centered_index();

        renderer.push_clip(viewport);

        // Center highlight band: a subtle fill behind the centered row, with a
        // top and bottom rule framing it.
        let band = Rectangle::new(
            Point::new(viewport.top_left.x, band_top),
            Size::new(viewport.size.width, self.item_height),
        );
        renderer.fill_rect(band, theme.secondary.base)?;
        renderer.stroke_line(
            Point::new(viewport.top_left.x, band_top),
            Point::new(viewport.top_left.x + viewport.size.width as i32, band_top),
            theme.accent.base,
            1,
        )?;
        let band_bot = band_top + ih;
        renderer.stroke_line(
            Point::new(viewport.top_left.x, band_bot),
            Point::new(viewport.top_left.x + viewport.size.width as i32, band_bot),
            theme.accent.base,
            1,
        )?;

        // Each option's row top in screen space: leading pad, minus scroll.
        for (i, label) in self.options.iter().enumerate() {
            let row_top = band_top + i as i32 * ih - off;
            // Cull rows fully outside the viewport.
            if row_top + ih <= viewport.top_left.y
                || row_top >= viewport.top_left.y + viewport.size.height as i32
            {
                continue;
            }
            let baseline_y = row_top + ih / 2 + glyph_h / 3;
            let color = if i == centered {
                theme.background.on_base
            } else {
                // Dim non-centered rows toward the divider tone.
                theme.background.divider
            };
            renderer.draw_text(
                label,
                Point::new(center_x, baseline_y),
                font,
                color,
                EgAlignment::Center,
            )?;
        }

        renderer.pop_clip();
        Ok(())
    }
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Roller<'a, C, M> {
    /// The option index currently under the highlight band for a given host
    /// [`ScrollState`] and `item_height`. The host calls this in its `update()`
    /// after applying a [`ScrollMsg`] (or after a [`ScrollState::tick`]) to
    /// learn which option is centered, without rebuilding the widget tree.
    ///
    /// `item_height` must match the value configured on the widget (default
    /// [`DEFAULT_ITEM_HEIGHT`]).
    #[must_use]
    pub fn centered_for(state: &ScrollState, item_height: u32, count: usize) -> usize {
        if count == 0 {
            return 0;
        }
        let off = scroll_core::render_offset(*state, ScrollDirection::Vertical).y;
        let ih = item_height.max(1) as i32;
        let raw = (off + ih / 2).div_euclid(ih);
        raw.clamp(0, count as i32 - 1) as usize
    }

    /// Build the [`on_select`](Roller::on_select) message for the
    /// currently-centered option when it differs from `previous`, or whenever
    /// the drum has settled to rest. Returns `None` if no `on_select` callback
    /// is set or the selection is unchanged mid-motion.
    ///
    /// The host typically calls this from `update()` after applying a
    /// [`ScrollMsg`]/`tick` so the selection follows the drum as it spins and
    /// commits when it settles.
    #[must_use]
    pub fn select_msg(&self, previous: usize) -> Option<M> {
        let now = self.centered_index();
        let settled = self.state.phase == GesturePhase::Idle;
        if now != previous || settled {
            self.on_select.as_ref().map(|f| f(now))
        } else {
            None
        }
    }
}
