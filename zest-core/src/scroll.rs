//! LVGL-style scroll state + math, living next to [`layout`](crate::layout).
//!
//! Widgets in `zest` are transient: the [`Runtime`](crate::runtime::Runtime)
//! rebuilds the whole tree from `view(&self)` every frame, so a container
//! cannot itself remember a drag origin or a fling velocity. All cross-frame
//! scroll/gesture state therefore lives in a single host-owned value —
//! [`ScrollState`] — which the screen stores as a field and passes by
//! reference into `view`. Containers only *read* it during measure/arrange/
//! draw and *emit* [`ScrollMsg`] in `handle_touch`; the owned copy is mutated
//! only in `update()` via [`ScrollState::apply`] and [`ScrollState::tick`].
//!
//! Touch events carry no timestamp (see [`event`](crate::event)), so velocity
//! for fling is sampled in `update()` from `embassy_time::Instant` deltas the
//! caller passes as `now_ms`, never inside `handle_touch`. Momentum is driven
//! by a self-rescheduling [`tick_task`] (a 16 ms `Timer`-delayed [`Task`]),
//! not a subscription, so the animation cleanly stops when it settles.
//!
//! The math deliberately avoids `libm`: the tap-vs-scroll threshold uses
//! Manhattan distance, and friction is a per-frame-constant multiply (no
//! `powf`/`sqrt`), valid because the tick `dt` is approximately constant at
//! 16 ms.

use crate::application::Task;
use alloc::vec::Vec;
use embassy_time::{Duration, Timer};
use embedded_graphics::prelude::*;

/// Pixel distance a finger must travel before a press is reinterpreted as a
/// scroll (Manhattan distance, `|dx| + |dy|`).
pub const SCROLL_THRESHOLD: i32 = 8;
/// Velocity multiplier applied each animation frame during a fling.
pub const FRICTION: f32 = 0.95;
/// Minimum fling speed (px/s, per axis) below which a fling becomes a spring.
pub const MIN_FLING_V: f32 = 20.0;
/// Maximum fling speed (px/s, per axis) the release sample is capped to.
pub const MAX_FLING_V: f32 = 4000.0;
/// Rubber-band resistance coefficient: larger → stiffer over-scroll.
pub const RUBBER_C: f32 = 0.55;
/// Spring stiffness: fraction of the remaining distance closed each frame.
pub const SPRING_K: f32 = 0.25;
/// Animation frame period in milliseconds (~60 fps).
pub const TICK_MS: u64 = 16;

/// Which axes a container scrolls on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Scroll vertically only.
    Vertical,
    /// Scroll horizontally only.
    Horizontal,
    /// Scroll on both axes.
    Both,
    /// No scrolling; the container does not pan.
    None,
}

impl ScrollDirection {
    /// True if vertical panning is permitted.
    #[must_use]
    pub fn scrolls_y(self) -> bool {
        matches!(self, ScrollDirection::Vertical | ScrollDirection::Both)
    }

    /// True if horizontal panning is permitted.
    #[must_use]
    pub fn scrolls_x(self) -> bool {
        matches!(self, ScrollDirection::Horizontal | ScrollDirection::Both)
    }
}

/// When the scrollbar thumb is visible.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScrollbarMode {
    /// Never draw the scrollbar.
    Off,
    /// Always draw the scrollbar.
    On,
    /// Draw the scrollbar only when content overflows the viewport.
    Auto,
    /// Draw the scrollbar only while a gesture or animation is active.
    Active,
}

/// How scrolling settles to content boundaries.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapMode {
    /// No snapping; settle at the released offset (clamped).
    None,
    /// Snap a child's leading edge to the viewport's leading edge.
    Start,
    /// Snap a child's center to the viewport's center.
    Center,
    /// Snap a child's trailing edge to the viewport's trailing edge.
    End,
}

/// Gesture / animation phase of a [`ScrollState`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GesturePhase {
    /// No interaction; offset is static.
    Idle,
    /// Finger is down but has not yet crossed the scroll threshold.
    Pressing,
    /// Finger is down and panning the content 1:1.
    Dragging,
    /// Finger released; coasting under friction.
    Flinging,
    /// Settling toward an edge or snap target via spring.
    Springing,
}

/// Cross-frame scroll/gesture/velocity state owned by the host screen.
///
/// `Copy` so the `scroll_core` engine can take it by value (see this module's
/// header for why).
#[derive(Copy, Clone, Debug)]
pub struct ScrollState {
    /// Current committed scroll offset in pixels (subtracted from children).
    pub offset: Point,
    /// Sub-pixel integrator for smooth fling/spring motion.
    pub accum: (f32, f32),
    /// Current gesture/animation phase.
    pub phase: GesturePhase,
    /// Screen point where the active press began.
    pub press_origin: Point,
    /// `offset` captured at the moment of press (drag is relative to this).
    pub offset_at_press: Point,
    /// Most recent point seen during the gesture.
    pub last_point: Point,
    /// Current velocity in px/s, sampled at release.
    pub velocity: (f32, f32),
    /// Point of the last velocity sample.
    pub last_sample_point: Point,
    /// Millisecond timestamp of the last velocity sample.
    pub last_sample_ms: u64,
    /// Spring destination while [`GesturePhase::Springing`].
    pub spring_target: Point,
    /// Cached maximum scrollable offset (`max(content - viewport, 0)`).
    pub max_offset: Point,
    /// Cached content size from the last layout pass.
    pub content: Size,
    /// Cached viewport size from the last layout pass.
    pub viewport: Size,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    /// A fresh, idle scroll state at offset zero.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            offset: Point::new(0, 0),
            accum: (0.0, 0.0),
            phase: GesturePhase::Idle,
            press_origin: Point::new(0, 0),
            offset_at_press: Point::new(0, 0),
            last_point: Point::new(0, 0),
            velocity: (0.0, 0.0),
            last_sample_point: Point::new(0, 0),
            last_sample_ms: 0,
            spring_target: Point::new(0, 0),
            max_offset: Point::new(0, 0),
            content: Size::new(0, 0),
            viewport: Size::new(0, 0),
        }
    }

    /// True while coasting ([`GesturePhase::Flinging`]) or settling
    /// ([`GesturePhase::Springing`]) — i.e. the host should keep ticking.
    #[must_use]
    pub fn is_animating(&self) -> bool {
        matches!(self.phase, GesturePhase::Flinging | GesturePhase::Springing)
    }

    /// True while any interaction is in progress (phase is not idle).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.phase != GesturePhase::Idle
    }

    /// Record the cached geometry from a [`ScrollMsg`] so subsequent math
    /// (clamp/rubber-band/spring) needs no layout access.
    fn cache_geometry(&mut self, content: Size, viewport: Size) {
        self.content = content;
        self.viewport = viewport;
        self.max_offset = Point::new(
            (content.width as i32 - viewport.width as i32).max(0),
            (content.height as i32 - viewport.height as i32).max(0),
        );
    }

    /// Clamp `offset` into the valid `[0, max_offset]` range on both axes.
    #[must_use]
    pub fn clamp_offset(&self, offset: Point) -> Point {
        Point::new(
            offset.x.clamp(0, self.max_offset.x),
            offset.y.clamp(0, self.max_offset.y),
        )
    }

    /// Apply rubber-band resistance to a raw drag offset: any overshoot past
    /// `[0, max_offset]` is compressed by `resist(o, dim) = o*dim/(dim+o*C)`.
    #[must_use]
    pub fn rubber_band(&self, offset: Point) -> Point {
        Point::new(
            rubber_axis(offset.x, self.max_offset.x, self.viewport.width),
            rubber_axis(offset.y, self.max_offset.y, self.viewport.height),
        )
    }

    /// Nearest snap line to `offset` among `lines`, clamped to range. Empty
    /// `lines` (or [`SnapMode::None`]) returns the clamped `offset`.
    #[must_use]
    pub fn nearest_snap(&self, offset: Point, snap: SnapMode, snap_lines: &[i32]) -> Point {
        let clamped = self.clamp_offset(offset);
        if snap == SnapMode::None || snap_lines.is_empty() {
            return clamped;
        }
        // Snap lines apply to the scrolling axis. Vertical containers store
        // line values as y offsets; horizontal as x. We pick whichever axis
        // actually overflows (the one with a non-zero max).
        if self.max_offset.y > 0 {
            let target = nearest(clamped.y, snap_lines).clamp(0, self.max_offset.y);
            Point::new(clamped.x, target)
        } else if self.max_offset.x > 0 {
            let target = nearest(clamped.x, snap_lines).clamp(0, self.max_offset.x);
            Point::new(target, clamped.y)
        } else {
            clamped
        }
    }

    /// Single dispatch over a [`ScrollMsg`]; `now_ms` is the caller's
    /// millisecond clock (from `embassy_time::Instant`) used for velocity
    /// sampling. Mutates the state in place.
    pub fn apply(&mut self, msg: ScrollMsg, now_ms: u64) {
        match msg {
            ScrollMsg::Press {
                point,
                content,
                viewport,
            } => self.on_press(point, content, viewport, now_ms),
            ScrollMsg::DragTo {
                point,
                content,
                viewport,
            } => self.on_move(point, content, viewport, now_ms),
            ScrollMsg::Release {
                point,
                content,
                viewport,
                snap_lines,
            } => self.on_release(point, content, viewport, &snap_lines, now_ms),
        }
    }

    fn on_press(&mut self, point: Point, content: Size, viewport: Size, now_ms: u64) {
        self.cache_geometry(content, viewport);
        // Reset all gesture state so a prior fling/spring can't leak in.
        self.phase = GesturePhase::Pressing;
        self.press_origin = point;
        self.offset_at_press = self.clamp_offset(self.offset);
        self.offset = self.offset_at_press;
        self.last_point = point;
        self.velocity = (0.0, 0.0);
        self.accum = (0.0, 0.0);
        self.last_sample_point = point;
        self.last_sample_ms = now_ms;
    }

    fn on_move(&mut self, point: Point, content: Size, viewport: Size, now_ms: u64) {
        self.cache_geometry(content, viewport);
        if self.phase == GesturePhase::Idle {
            // Defensive: a move with no press — treat as a press.
            self.on_press(point, content, viewport, now_ms);
            return;
        }
        self.phase = GesturePhase::Dragging;
        // Pan relative to the offset captured at press. A drag DOWN
        // (point.y increases) reveals earlier content → offset decreases.
        let raw = Point::new(
            self.offset_at_press.x - (point.x - self.press_origin.x),
            self.offset_at_press.y - (point.y - self.press_origin.y),
        );
        self.offset = self.rubber_band(raw);
        // Velocity sample for the eventual fling: px/s over the elapsed dt.
        let dt = now_ms.saturating_sub(self.last_sample_ms);
        if dt > 0 {
            let dx = (point.x - self.last_sample_point.x) as f32;
            let dy = (point.y - self.last_sample_point.y) as f32;
            let s = 1000.0 / dt as f32;
            // Finger motion is opposite to offset motion → negate.
            self.velocity = (clamp_v(-dx * s), clamp_v(-dy * s));
            self.last_sample_point = point;
            self.last_sample_ms = now_ms;
        }
        self.last_point = point;
    }

    fn on_release(
        &mut self,
        point: Point,
        content: Size,
        viewport: Size,
        snap_lines: &[i32],
        now_ms: u64,
    ) {
        self.cache_geometry(content, viewport);
        let _ = (point, now_ms);
        self.accum = (0.0, 0.0);

        let past_edge = self.offset != self.clamp_offset(self.offset);
        let fast = self.velocity.0.abs() >= MIN_FLING_V || self.velocity.1.abs() >= MIN_FLING_V;

        if !past_edge && fast {
            self.phase = GesturePhase::Flinging;
        } else {
            // Settle: spring toward the nearest snap line, else the edge.
            self.spring_target =
                self.nearest_snap(self.offset, snap_mode_from(snap_lines), snap_lines);
            self.phase = GesturePhase::Springing;
        }
    }

    /// Advance one animation frame. `dt_ms` is elapsed milliseconds since the
    /// last tick; `snap`/`snap_lines` describe the settle target. Transitions
    /// Flinging → Springing → Idle and updates `offset` in place.
    pub fn tick(&mut self, dt_ms: u32, snap: SnapMode, snap_lines: &[i32]) {
        let dt = (dt_ms.max(1) as f32) / 1000.0;
        match self.phase {
            GesturePhase::Flinging => {
                // Integrate position with sub-pixel accumulator.
                self.accum.0 += self.velocity.0 * dt;
                self.accum.1 += self.velocity.1 * dt;
                let step = Point::new(self.accum.0 as i32, self.accum.1 as i32);
                self.accum.0 -= step.x as f32;
                self.accum.1 -= step.y as f32;
                self.offset += step;
                // Decay (per-frame-constant friction — no powf needed).
                self.velocity.0 *= FRICTION;
                self.velocity.1 *= FRICTION;

                let past_edge = self.offset != self.clamp_offset(self.offset);
                let slow =
                    self.velocity.0.abs() < MIN_FLING_V && self.velocity.1.abs() < MIN_FLING_V;
                if past_edge || slow {
                    self.spring_target = self.nearest_snap(self.offset, snap, snap_lines);
                    self.velocity = (0.0, 0.0);
                    self.accum = (0.0, 0.0);
                    self.phase = GesturePhase::Springing;
                }
            }
            GesturePhase::Springing => {
                let dx = (self.spring_target.x - self.offset.x) as f32;
                let dy = (self.spring_target.y - self.offset.y) as f32;
                if dx.abs() < 1.0 && dy.abs() < 1.0 {
                    self.offset = self.spring_target;
                    self.accum = (0.0, 0.0);
                    self.velocity = (0.0, 0.0);
                    self.phase = GesturePhase::Idle;
                } else {
                    self.accum.0 += dx * SPRING_K;
                    self.accum.1 += dy * SPRING_K;
                    let step = Point::new(self.accum.0 as i32, self.accum.1 as i32);
                    self.accum.0 -= step.x as f32;
                    self.accum.1 -= step.y as f32;
                    // Guarantee progress even when the per-frame step rounds
                    // to zero, so the spring always reaches its target.
                    let step = Point::new(nudge(step.x, dx), nudge(step.y, dy));
                    self.offset += step;
                }
            }
            _ => {}
        }
    }
}

/// Message a scrollable container emits in `handle_touch`; the host applies it
/// in `update()`. Carries the geometry (and snap lines on release) so the
/// owned [`ScrollState`] can integrate without touching layout.
#[derive(Clone, Debug)]
pub enum ScrollMsg {
    /// Finger landed inside the viewport.
    Press {
        /// Touch point in screen space.
        point: Point,
        /// Content size measured this frame.
        content: Size,
        /// Viewport size this frame.
        viewport: Size,
    },
    /// Finger moved while dragging (pan).
    DragTo {
        /// Touch point in screen space.
        point: Point,
        /// Content size measured this frame.
        content: Size,
        /// Viewport size this frame.
        viewport: Size,
    },
    /// Finger released after a drag (seed fling / spring-back).
    Release {
        /// Touch point in screen space.
        point: Point,
        /// Content size measured this frame.
        content: Size,
        /// Viewport size this frame.
        viewport: Size,
        /// Candidate snap offsets from child positions (empty = no snap).
        snap_lines: Vec<i32>,
    },
}

/// Self-rescheduling animation clock: a [`Task`] that waits one frame
/// (~16 ms) and then yields `msg`, so the host's `ScrollTick` arm can call
/// [`ScrollState::tick`] and re-arm while [`ScrollState::is_animating`].
///
/// This drives momentum without a perpetual `time::every` subscription: the
/// loop stops the moment the host stops returning another `tick_task`.
#[must_use]
pub fn tick_task<M: 'static>(msg: M) -> Task<M> {
    Task::perform(async move {
        Timer::after(Duration::from_millis(TICK_MS)).await;
        Some(msg)
    })
}

// ---- free helpers ------------------------------------------------------

/// Cap a velocity component to `±MAX_FLING_V`.
fn clamp_v(v: f32) -> f32 {
    v.clamp(-MAX_FLING_V, MAX_FLING_V)
}

/// Rubber-band one axis: pass-through inside `[0, max]`, compressed outside.
fn rubber_axis(value: i32, max: i32, dim: u32) -> i32 {
    if value < 0 {
        -resist(-value, dim)
    } else if value > max {
        max + resist(value - max, dim)
    } else {
        value
    }
}

/// Diminishing-returns resistance: `o*dim/(dim + o*RUBBER_C)`.
fn resist(overshoot: i32, dim: u32) -> i32 {
    let o = overshoot as f32;
    let d = (dim.max(1)) as f32;
    (o * d / (d + o * RUBBER_C)) as i32
}

/// Nearest value in `lines` to `value` (linear scan; lists are small).
fn nearest(value: i32, lines: &[i32]) -> i32 {
    let mut best = lines[0];
    let mut best_d = (value - best).abs();
    for &l in &lines[1..] {
        let d = (value - l).abs();
        if d < best_d {
            best_d = d;
            best = l;
        }
    }
    best
}

/// Ensure a rounded spring step moves at least one pixel toward the target
/// (sign of `delta`) so settling never stalls.
fn nudge(step: i32, delta: f32) -> i32 {
    if step != 0 {
        step
    } else if delta > 0.5 {
        1
    } else if delta < -0.5 {
        -1
    } else {
        0
    }
}

/// Snapping is on iff snap lines exist. Returns `Start` as a generic "snap"
/// marker — the edge/center/end geometry is already baked into the line values
/// by `scroll_core::snap_lines`.
fn snap_mode_from(snap_lines: &[i32]) -> SnapMode {
    if snap_lines.is_empty() {
        SnapMode::None
    } else {
        SnapMode::Start
    }
}
