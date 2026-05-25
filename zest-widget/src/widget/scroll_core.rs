//! Shared scrolling engine reused by every scrollable container.
//!
//! The logic lives here as free functions so it is not triplicated across
//! `Column`/`Row`/`Container`/`Grid`. Because [`ScrollState`] is `Copy`, each
//! function takes the state *by value* plus a `forward` closure for routing a
//! touch into the container's children. That sidesteps the borrow checker
//! aliasing `&self.children` with `&self.scroll` inside a container method.
//!
//! Responsibilities:
//! - [`route_touch`] — the tap-vs-scroll gesture decision flow.
//! - [`render_offset`] — the pixel offset a container subtracts from child
//!   positions in `arrange` (rubber-banded while dragging, clamped otherwise).
//! - [`snap_lines`] — candidate snap offsets derived from child rectangles.
//! - [`draw_scrollbars`] — track + proportional thumb for both axes, honoring
//!   every [`ScrollbarMode`].

use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use zest_core::{
    GesturePhase, RenderError, Renderer, ScrollDirection, ScrollMsg, ScrollState, ScrollbarMode,
    SnapMode, TouchPhase,
};
use zest_theme::Theme;

/// Width/height (px) of the scrollbar gutter and thumb.
pub const SCROLLBAR_W: u32 = 8;

/// The pixel offset a container subtracts from each child's position during
/// `arrange`. While dragging the raw offset is rubber-banded so over-scroll
/// stretches; otherwise it is clamped to the valid range.
#[must_use]
pub fn render_offset(state: ScrollState, dir: ScrollDirection) -> Point {
    let off = if state.phase == GesturePhase::Dragging {
        state.rubber_band(state.offset)
    } else {
        // During fling/spring the offset may legitimately sit past an edge
        // (rubber-band overshoot before spring-back); keep it as-is so the
        // stretch is visible, only forcing axes the container doesn't scroll.
        state.offset
    };
    Point::new(
        if dir.scrolls_x() { off.x } else { 0 },
        if dir.scrolls_y() { off.y } else { 0 },
    )
}

/// Run the LVGL-style gesture flow for one touch event, returning the host
/// message to emit (if any). `forward` routes a touch into the container's
/// children (typically the reverse child iteration) and reports whether a
/// child consumed it.
///
/// Flow:
/// - `Down`: forward to the child (so a button can highlight) **and** emit
///   [`ScrollMsg::Press`].
/// - `Moved` while `Pressing`: if the finger has crossed the threshold, begin
///   dragging (emit [`ScrollMsg::DragTo`], stop forwarding); else forward.
/// - `Moved` while `Dragging`: pan (emit [`ScrollMsg::DragTo`]).
/// - `Up` while `Pressing`: it was a tap — forward to the child; if the child
///   does nothing, emit [`ScrollMsg::Release`] to reset the phase.
/// - `Up` while `Dragging`: emit [`ScrollMsg::Release`] (seed fling/spring).
#[allow(clippy::too_many_arguments)]
pub fn route_touch<M, F, S>(
    state: ScrollState,
    dir: ScrollDirection,
    viewport: Rectangle,
    content: Size,
    point: Point,
    phase: TouchPhase,
    snap_lines: &[i32],
    on_scroll: Option<&S>,
    mut forward: F,
) -> Option<M>
where
    F: FnMut(Point, TouchPhase) -> Option<M>,
    S: Fn(ScrollMsg) -> M + ?Sized,
{
    let inside = rect_contains(viewport, point);
    let emit = |sm: ScrollMsg| -> Option<M> { on_scroll.map(|f| f(sm)) };

    match phase {
        TouchPhase::Down => {
            // Forward first so a child (button) can mark itself pressed, then
            // begin the press gesture regardless of whether the child took it.
            let child_msg = if inside { forward(point, phase) } else { None };
            if inside {
                let press = emit(ScrollMsg::Press {
                    point,
                    content,
                    viewport: viewport.size,
                });
                // Prefer the child message if any (e.g. a Down that produced
                // something), else carry the Press so phase advances.
                child_msg.or(press)
            } else {
                child_msg
            }
        }
        TouchPhase::Moved => match state.phase {
            GesturePhase::Pressing => {
                let crossed = crossed_threshold(state.press_origin, point, dir);
                if crossed {
                    emit(ScrollMsg::DragTo {
                        point,
                        content,
                        viewport: viewport.size,
                    })
                } else {
                    forward(point, phase)
                }
            }
            GesturePhase::Dragging => emit(ScrollMsg::DragTo {
                point,
                content,
                viewport: viewport.size,
            }),
            _ => {
                if inside {
                    forward(point, phase)
                } else {
                    None
                }
            }
        },
        TouchPhase::Up => match state.phase {
            GesturePhase::Pressing => {
                // Tap: let the child fire. If it does, that wins; otherwise
                // emit Release so the phase returns to Idle.
                let child_msg = forward(point, phase);
                child_msg.or_else(|| {
                    emit(ScrollMsg::Release {
                        point,
                        content,
                        viewport: viewport.size,
                        snap_lines: snap_lines.to_vec(),
                    })
                })
            }
            GesturePhase::Dragging => emit(ScrollMsg::Release {
                point,
                content,
                viewport: viewport.size,
                snap_lines: snap_lines.to_vec(),
            }),
            _ => {
                if inside {
                    forward(point, phase)
                } else {
                    None
                }
            }
        },
    }
}

/// Whether the finger has moved far enough from `origin` (Manhattan distance,
/// restricted to the scrolling axes) to be treated as a scroll rather than a
/// tap.
#[must_use]
pub fn crossed_threshold(origin: Point, point: Point, dir: ScrollDirection) -> bool {
    let dx = if dir.scrolls_x() {
        (point.x - origin.x).abs()
    } else {
        0
    };
    let dy = if dir.scrolls_y() {
        (point.y - origin.y).abs()
    } else {
        0
    };
    dx + dy >= zest_core::scroll::SCROLL_THRESHOLD
}

/// Candidate snap offsets (in offset-space, i.e. valid `ScrollState::offset`
/// values) derived from `child_rects`. `origin` is the viewport's top-left,
/// `offset` the scroll offset already applied to the arranged children this
/// frame (added back so the result is independent of the current scroll), and
/// `viewport` the viewport size. Returns offsets on the scrolling axis
/// selected by `dir`.
///
/// - [`SnapMode::Start`] — each child's leading edge aligns to the viewport
///   leading edge.
/// - [`SnapMode::Center`] — each child's center aligns to the viewport center.
/// - [`SnapMode::End`] — each child's trailing edge aligns to the viewport
///   trailing edge.
#[must_use]
pub fn snap_lines(
    child_rects: &[Rectangle],
    origin: Point,
    offset: Point,
    viewport: Size,
    dir: ScrollDirection,
    mode: SnapMode,
) -> Vec<i32> {
    if mode == SnapMode::None {
        return Vec::new();
    }
    let vertical = dir.scrolls_y();
    let mut out: Vec<i32> = Vec::with_capacity(child_rects.len());
    for r in child_rects {
        // Children are already shifted by `-offset`. The child's leading edge
        // in offset-space (the offset that puts this child at the viewport
        // leading edge) is `(top_left - origin) + offset`.
        let (lead, extent, vp) = if vertical {
            (
                r.top_left.y - origin.y + offset.y,
                r.size.height as i32,
                viewport.height as i32,
            )
        } else {
            (
                r.top_left.x - origin.x + offset.x,
                r.size.width as i32,
                viewport.width as i32,
            )
        };
        let line = match mode {
            SnapMode::Center => lead + extent / 2 - vp / 2,
            SnapMode::End => lead + extent - vp,
            // Start (and the unreachable None, filtered above) snap to the
            // leading edge.
            SnapMode::Start | SnapMode::None => lead,
        };
        out.push(line.max(0));
    }
    out
}

/// Draw scrollbar tracks/thumbs after the children have been clipped+drawn.
/// Honors all four [`ScrollbarMode`]s and both axes per `dir`. Track color is
/// `theme.background.divider`, thumb is `theme.accent.base`.
#[allow(clippy::too_many_arguments)]
pub fn draw_scrollbars<C: PixelColor>(
    renderer: &mut dyn Renderer<C>,
    theme: &Theme<'_, C>,
    state: ScrollState,
    mode: ScrollbarMode,
    dir: ScrollDirection,
    viewport: Rectangle,
    content: Size,
) -> Result<(), RenderError> {
    let want = |overflow: bool| match mode {
        ScrollbarMode::Off => false,
        ScrollbarMode::On => true,
        ScrollbarMode::Auto => overflow,
        ScrollbarMode::Active => overflow && state.is_active(),
    };

    let off = render_offset(state, dir);

    if dir.scrolls_y() {
        let overflow = content.height > viewport.size.height;
        if want(overflow) && overflow {
            let track = Rectangle::new(
                Point::new(
                    viewport.top_left.x + viewport.size.width.saturating_sub(SCROLLBAR_W) as i32,
                    viewport.top_left.y,
                ),
                Size::new(SCROLLBAR_W, viewport.size.height),
            );
            renderer.fill_rect(track, theme.background.divider)?;
            let thumb = thumb_rect(track, viewport.size.height, content.height, off.y, true);
            renderer.fill_rect(thumb, theme.accent.base)?;
        }
    }

    if dir.scrolls_x() {
        let overflow = content.width > viewport.size.width;
        if want(overflow) && overflow {
            let track = Rectangle::new(
                Point::new(
                    viewport.top_left.x,
                    viewport.top_left.y + viewport.size.height.saturating_sub(SCROLLBAR_W) as i32,
                ),
                Size::new(viewport.size.width, SCROLLBAR_W),
            );
            renderer.fill_rect(track, theme.background.divider)?;
            let thumb = thumb_rect(track, viewport.size.width, content.width, off.x, false);
            renderer.fill_rect(thumb, theme.accent.base)?;
        }
    }

    Ok(())
}

/// Thumb rectangle within `track`. `vp`/`content` are the relevant axis
/// extents, `offset` the current scroll on that axis, `vertical` selects the
/// axis. The thumb is sized to the visible fraction (minimum 8 px) and
/// positioned by the scroll fraction.
#[must_use]
pub fn thumb_rect(
    track: Rectangle,
    vp: u32,
    content: u32,
    offset: i32,
    vertical: bool,
) -> Rectangle {
    let track_len = if vertical {
        track.size.height
    } else {
        track.size.width
    };
    let max = (content as i32 - vp as i32).max(0);
    let visible_frac = vp as f32 / content as f32;
    let thumb_len = ((track_len as f32 * visible_frac).max(8.0)) as u32;
    let thumb_len = thumb_len.min(track_len);
    let scroll_frac = if max > 0 {
        (offset.clamp(0, max) as f32) / max as f32
    } else {
        0.0
    };
    let along = ((track_len.saturating_sub(thumb_len)) as f32 * scroll_frac) as i32;
    if vertical {
        Rectangle::new(
            Point::new(track.top_left.x + 2, track.top_left.y + along),
            Size::new(track.size.width.saturating_sub(4), thumb_len),
        )
    } else {
        Rectangle::new(
            Point::new(track.top_left.x + along, track.top_left.y + 2),
            Size::new(thumb_len, track.size.height.saturating_sub(4)),
        )
    }
}

/// Half-open hit test: `point` lies within `rect`.
#[must_use]
pub fn rect_contains(rect: Rectangle, point: Point) -> bool {
    let br = rect.top_left + Point::new(rect.size.width as i32, rect.size.height as i32);
    point.x >= rect.top_left.x && point.x < br.x && point.y >= rect.top_left.y && point.y < br.y
}
