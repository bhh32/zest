//! Vertical scroll viewport with explicit up/down buttons and a
//! scrollbar thumb indicator.
//!
//! The host owns `scroll_y` (immediate-mode style) and supplies it via
//! `.scroll_y(...)`. Tapping the up/down arrows fires `on_scroll(new_y)`
//! with the new clamped offset; the host updates its state and passes
//! the new value back next frame.
//!
//! Drag-to-scroll is intentionally out of scope here — it requires
//! persisting drag origin + initial scroll across frames, which needs
//! either runtime-level state augmentation or the tree-diff state model
//! (filed in `FOLLOW_UPS.md`). Scroll buttons cover the embedded UX.

use super::{Widget, element::Element};
use alloc::boxed::Box;
use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::PixelColor, prelude::*, primitives::Rectangle, text::Alignment,
};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::{ButtonCatalog, ButtonClass, Status, Theme};

const SCROLLBAR_W: u32 = 16;
const NAV_H: u32 = 20;
const DEFAULT_STEP: u32 = 40;
const _: () = ();

/// Vertical scrollable viewport holding one child.
pub struct Scrollable<'a, C: PixelColor, M: Clone> {
    rect: Rectangle,
    child: Element<'a, C, M>,
    content_h: u32,
    scroll_y: i32,
    step: u32,
    on_scroll: Option<Box<dyn Fn(i32) -> M + 'a>>,
    width: Length,
    height: Length,
    pressed_nav: Option<NavDir>,
    _phantom: PhantomData<C>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NavDir {
    Up,
    Down,
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Scrollable<'a, C, M> {
    /// Construct a scrollable wrapping `child`. Defaults: `scroll_y = 0`,
    /// step `40` pixels per arrow tap.
    pub fn new<W>(child: W) -> Self
    where
        W: Widget<C, M> + 'a,
    {
        Self {
            rect: Rectangle::zero(),
            child: Element::new(child),
            content_h: 0,
            scroll_y: 0,
            step: DEFAULT_STEP,
            on_scroll: None,
            width: Length::Fill,
            height: Length::Fill,
            pressed_nav: None,
            _phantom: PhantomData,
        }
    }

    /// Builder: current scroll offset in pixels. Clamped to
    /// `[0, max_scroll]` at draw/touch time.
    #[must_use]
    pub fn scroll_y(mut self, y: i32) -> Self {
        self.scroll_y = y;
        self
    }

    /// Builder: pixels scrolled per arrow tap.
    #[must_use]
    pub fn step(mut self, step: u32) -> Self {
        self.step = step.max(1);
        self
    }

    /// Builder: callback fired with the new (already clamped) scroll
    /// offset when an arrow is tapped.
    #[must_use]
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(i32) -> M + 'a,
    {
        self.on_scroll = Some(Box::new(f));
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

    fn content_rect(&self) -> Rectangle {
        Rectangle::new(
            self.rect.top_left,
            Size::new(
                self.rect.size.width.saturating_sub(SCROLLBAR_W),
                self.rect.size.height,
            ),
        )
    }

    fn up_rect(&self) -> Rectangle {
        Rectangle::new(
            self.rect.top_left
                + Point::new(self.rect.size.width.saturating_sub(SCROLLBAR_W) as i32, 0),
            Size::new(SCROLLBAR_W, NAV_H),
        )
    }

    fn down_rect(&self) -> Rectangle {
        Rectangle::new(
            self.rect.top_left
                + Point::new(
                    self.rect.size.width.saturating_sub(SCROLLBAR_W) as i32,
                    self.rect.size.height.saturating_sub(NAV_H) as i32,
                ),
            Size::new(SCROLLBAR_W, NAV_H),
        )
    }

    fn track_rect(&self) -> Rectangle {
        Rectangle::new(
            self.rect.top_left
                + Point::new(
                    self.rect.size.width.saturating_sub(SCROLLBAR_W) as i32,
                    NAV_H as i32,
                ),
            Size::new(
                SCROLLBAR_W,
                self.rect.size.height.saturating_sub(NAV_H * 2),
            ),
        )
    }

    fn max_scroll(&self) -> i32 {
        let viewport_h = self.rect.size.height as i32;
        let content = self.content_h as i32;
        (content - viewport_h).max(0)
    }

    fn clamped_scroll(&self) -> i32 {
        self.scroll_y.clamp(0, self.max_scroll())
    }

    fn hit_nav(&self, p: Point) -> Option<NavDir> {
        if rect_contains(self.up_rect(), p) {
            Some(NavDir::Up)
        } else if rect_contains(self.down_rect(), p) {
            Some(NavDir::Down)
        } else {
            None
        }
    }
}

fn rect_contains(r: Rectangle, p: Point) -> bool {
    let br = r.top_left + Point::new(r.size.width as i32, r.size.height as i32);
    p.x >= r.top_left.x && p.x < br.x && p.y >= r.top_left.y && p.y < br.y
}

impl<'a, C: PixelColor + 'a, M: Clone + 'a> Widget<C, M> for Scrollable<'a, C, M> {
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
        // Measure child against an unbounded height to learn its
        // intrinsic content height.
        let content_w = rect.size.width.saturating_sub(SCROLLBAR_W);
        let child_constraints =
            Constraints::loose(Size::new(content_w, zest_core::UNBOUNDED));
        let measured = self.child.measure(child_constraints);
        self.content_h = measured.height;
        let scroll = self.clamped_scroll();
        self.child.arrange(Rectangle::new(
            rect.top_left - Point::new(0, scroll),
            Size::new(content_w, self.content_h.max(rect.size.height)),
        ));
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, point: Point, phase: TouchPhase) -> Option<M> {
        match phase {
            TouchPhase::Down => {
                self.pressed_nav = self.hit_nav(point);
                if self.pressed_nav.is_some() {
                    return None;
                }
                if rect_contains(self.content_rect(), point) {
                    return self.child.handle_touch(point, phase);
                }
                None
            }
            TouchPhase::Moved => {
                if self.pressed_nav.is_some() {
                    if self.hit_nav(point) != self.pressed_nav {
                        self.pressed_nav = None;
                    }
                    return None;
                }
                if rect_contains(self.content_rect(), point) {
                    return self.child.handle_touch(point, phase);
                }
                None
            }
            TouchPhase::Up => {
                let nav_now = self.hit_nav(point);
                let pressed = self.pressed_nav.take();
                if let (Some(dir), Some(p)) = (nav_now, pressed) {
                    if dir == p {
                        let new_y = match dir {
                            NavDir::Up => self.scroll_y - self.step as i32,
                            NavDir::Down => self.scroll_y + self.step as i32,
                        }
                        .clamp(0, self.max_scroll());
                        if let Some(cb) = self.on_scroll.as_ref() {
                            return Some(cb(new_y));
                        }
                        return None;
                    }
                }
                if rect_contains(self.content_rect(), point) {
                    return self.child.handle_touch(point, phase);
                }
                None
            }
        }
    }

    fn mark_pressed(&mut self, point: Point) {
        if self.pressed_nav.is_none() {
            self.pressed_nav = self.hit_nav(point);
        }
        if self.pressed_nav.is_none() && rect_contains(self.content_rect(), point) {
            self.child.mark_pressed(point);
        }
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let viewport = self.content_rect();
        renderer.push_clip(viewport);
        self.child.draw(renderer, theme)?;
        renderer.pop_clip();

        // Scrollbar column: up button | track + thumb | down button.
        let up = self.up_rect();
        let down = self.down_rect();
        let track = self.track_rect();
        let scroll = self.clamped_scroll();
        let max = self.max_scroll();
        let at_top = scroll <= 0;
        let at_bot = scroll >= max;
        let up_enabled = self.on_scroll.is_some() && !at_top;
        let down_enabled = self.on_scroll.is_some() && !at_bot;

        let up_status = if !up_enabled {
            Status::Disabled
        } else if self.pressed_nav == Some(NavDir::Up) {
            Status::Pressed
        } else {
            Status::Active
        };
        let down_status = if !down_enabled {
            Status::Disabled
        } else if self.pressed_nav == Some(NavDir::Down) {
            Status::Pressed
        } else {
            Status::Active
        };
        let up_app = theme.button(ButtonClass::Standard, up_status);
        let down_app = theme.button(ButtonClass::Standard, down_status);

        if let Some(bg) = up_app.background {
            renderer.fill_rect(up, bg)?;
        }
        if let Some(border) = up_app.border {
            renderer.stroke_rect(up, border)?;
        }
        if let Some(bg) = down_app.background {
            renderer.fill_rect(down, bg)?;
        }
        if let Some(border) = down_app.border {
            renderer.stroke_rect(down, border)?;
        }
        let baseline_offset = (theme.typography.body.character_size.height / 3) as i32;
        renderer.draw_text(
            "^",
            Point::new(
                up.top_left.x + (up.size.width / 2) as i32,
                up.top_left.y + (up.size.height / 2) as i32 + baseline_offset,
            ),
            theme.typography.body,
            up_app.text,
            Alignment::Center,
        )?;
        renderer.draw_text(
            "v",
            Point::new(
                down.top_left.x + (down.size.width / 2) as i32,
                down.top_left.y + (down.size.height / 2) as i32 + baseline_offset,
            ),
            theme.typography.body,
            down_app.text,
            Alignment::Center,
        )?;

        // Track background (subtle).
        renderer.fill_rect(track, theme.background.divider)?;
        // Thumb: proportion = viewport_h / content_h, position = scroll/max
        if self.content_h > self.rect.size.height && track.size.height > 4 {
            let visible_frac = self.rect.size.height as f32 / self.content_h as f32;
            let thumb_h = ((track.size.height as f32 * visible_frac).max(8.0)) as u32;
            let thumb_h = thumb_h.min(track.size.height);
            let scroll_frac = if max > 0 {
                scroll as f32 / max as f32
            } else {
                0.0
            };
            let thumb_y = track.top_left.y
                + ((track.size.height.saturating_sub(thumb_h)) as f32 * scroll_frac) as i32;
            let thumb_rect = Rectangle::new(
                Point::new(track.top_left.x + 2, thumb_y),
                Size::new(track.size.width.saturating_sub(4), thumb_h),
            );
            renderer.fill_rect(thumb_rect, theme.accent.base)?;
        }

        Ok(())
    }
}
