//! Desktop simulator [`Platform`] for `zest`.
//!
//! Shapes are rendered with anti-aliasing via `tiny-skia` into an
//! RGBA pixmap, then converted to RGB565 for the SDL2-backed
//! `embedded-graphics-simulator` window. Text stays pixel-perfect
//! (bitmap `MonoFont`) to match the on-device look.

use embassy_time::{Duration, Timer};
use embedded_graphics::{
    Pixel,
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    text::{Alignment, Text},
};
use embedded_graphics_simulator::{
    OutputSettings, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
    sdl2::{Keycode, MouseButton, MouseWheelDirection},
};
use std::{convert::Infallible, vec::Vec};
use tiny_skia::{
    Color as SkColor, FillRule, Mask, Paint, PathBuilder, Pixmap, Rect as SkRect, Stroke, Transform,
};
use zest_core::{
    ButtonState, DirtyRegion, EncoderEvent, InputEvent, Key, KeyEvent, Platform,
    PlatformCapabilities, RenderError, Renderer, TouchEvent, TouchPhase,
};

/// Default display width — matches the CYD R3 panel.
pub const DEFAULT_WIDTH: u32 = 320;
/// Default display height — matches the CYD R3 panel.
pub const DEFAULT_HEIGHT: u32 = 240;
/// Default window scaling factor.
pub const DEFAULT_SCALE: u32 = 2;
/// Default per-pixel gap. Anti-aliased rendering supplies its own
/// smoothness, so the default is `0` (sharp upscale).
pub const DEFAULT_PIXEL_SPACING: u32 = 0;
/// Default event-poll interval in milliseconds (≈ 60 fps).
pub const DEFAULT_POLL_MS: u64 = 16;

/// Configurable builder for [`SimulatorPlatform`].
pub struct SimulatorPlatformBuilder {
    title: String,
    size: Size,
    scale: u32,
    pixel_spacing: u32,
    poll_ms: u64,
    show_dirty: bool,
}

impl SimulatorPlatformBuilder {
    /// New builder with the given window title; defaults otherwise.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            size: Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
            scale: DEFAULT_SCALE,
            pixel_spacing: DEFAULT_PIXEL_SPACING,
            poll_ms: DEFAULT_POLL_MS,
            show_dirty: false,
        }
    }

    /// Override display size (default: 320×240).
    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Override window scale (default: 2).
    #[must_use]
    pub fn scale(mut self, scale: u32) -> Self {
        self.scale = scale;
        self
    }

    /// Override per-pixel gap (default: 0).
    ///
    /// `1` mimics a real LCD's pixel grid by drawing each emulated
    /// pixel as a `scale × scale` block followed by a 1-px gap.
    /// Combined with anti-aliased shapes this can look over-busy;
    /// keep it `0` for a flat upscale.
    #[must_use]
    pub fn pixel_spacing(mut self, pixel_spacing: u32) -> Self {
        self.pixel_spacing = pixel_spacing;
        self
    }

    /// Override SDL event-poll interval in ms (default: 16 = 60 fps).
    #[must_use]
    pub fn poll_ms(mut self, poll_ms: u64) -> Self {
        self.poll_ms = poll_ms;
        self
    }

    /// Draw an outline over dirty rectangles after each partial redraw.
    #[must_use]
    pub fn show_dirty(mut self, show_dirty: bool) -> Self {
        self.show_dirty = show_dirty;
        self
    }

    /// Build.
    #[must_use]
    pub fn build(self) -> SimulatorPlatform {
        let settings: OutputSettings = OutputSettingsBuilder::new()
            .scale(self.scale)
            .pixel_spacing(self.pixel_spacing)
            .build();
        let pixmap = Pixmap::new(self.size.width, self.size.height).expect("non-zero pixmap size");
        SimulatorPlatform {
            display: SimulatorDisplay::new(self.size),
            window: Window::new(&self.title, &settings),
            pixmap,
            size: self.size,
            mouse_down: false,
            poll_ms: self.poll_ms,
            show_dirty: self.show_dirty,
            pending: None,
        }
    }
}

/// `Platform` implementation backed by SDL2.
pub struct SimulatorPlatform {
    display: SimulatorDisplay<Rgb565>,
    window: Window,
    pixmap: Pixmap,
    size: Size,
    mouse_down: bool,
    poll_ms: u64,
    show_dirty: bool,
    /// One-slot buffer holding a discrete Down/Up event that arrived in the
    /// same poll batch as coalesced moves, so it is delivered on the next
    /// `next_event` call rather than dropped.
    pending: Option<InputEvent>,
}

impl SimulatorPlatform {
    /// Convenience: default 320×240 RGB565 at 2× scale, 60 fps polling.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        SimulatorPlatformBuilder::new(title).build()
    }

    /// Builder for custom configuration.
    #[must_use]
    pub fn builder(title: impl Into<String>) -> SimulatorPlatformBuilder {
        SimulatorPlatformBuilder::new(title)
    }

    fn key_event(keycode: Keycode, repeat: bool, pressed: bool) -> Option<InputEvent> {
        let state = if pressed {
            if repeat {
                ButtonState::Repeated
            } else {
                ButtonState::Pressed
            }
        } else {
            ButtonState::Released
        };

        let key = match keycode {
            Keycode::Return | Keycode::KpEnter => Key::Enter,
            Keycode::Tab => Key::Tab,
            Keycode::Backspace => Key::Backspace,
            Keycode::Delete => Key::Delete,
            Keycode::Left => Key::Left,
            Keycode::Right => Key::Right,
            Keycode::Up => Key::Up,
            Keycode::Down => Key::Down,
            Keycode::Home => Key::Home,
            Keycode::End => Key::End,
            Keycode::PageUp => Key::PageUp,
            Keycode::PageDown => Key::PageDown,
            Keycode::Space => Key::Char(' '),
            Keycode::A => Key::Char('a'),
            Keycode::B => Key::Char('b'),
            Keycode::C => Key::Char('c'),
            Keycode::D => Key::Char('d'),
            Keycode::E => Key::Char('e'),
            Keycode::F => Key::Char('f'),
            Keycode::G => Key::Char('g'),
            Keycode::H => Key::Char('h'),
            Keycode::I => Key::Char('i'),
            Keycode::J => Key::Char('j'),
            Keycode::K => Key::Char('k'),
            Keycode::L => Key::Char('l'),
            Keycode::M => Key::Char('m'),
            Keycode::N => Key::Char('n'),
            Keycode::O => Key::Char('o'),
            Keycode::P => Key::Char('p'),
            Keycode::Q => Key::Char('q'),
            Keycode::R => Key::Char('r'),
            Keycode::S => Key::Char('s'),
            Keycode::T => Key::Char('t'),
            Keycode::U => Key::Char('u'),
            Keycode::V => Key::Char('v'),
            Keycode::W => Key::Char('w'),
            Keycode::X => Key::Char('x'),
            Keycode::Y => Key::Char('y'),
            Keycode::Z => Key::Char('z'),
            Keycode::Num0 | Keycode::Kp0 => Key::Char('0'),
            Keycode::Num1 | Keycode::Kp1 => Key::Char('1'),
            Keycode::Num2 | Keycode::Kp2 => Key::Char('2'),
            Keycode::Num3 | Keycode::Kp3 => Key::Char('3'),
            Keycode::Num4 | Keycode::Kp4 => Key::Char('4'),
            Keycode::Num5 | Keycode::Kp5 => Key::Char('5'),
            Keycode::Num6 | Keycode::Kp6 => Key::Char('6'),
            Keycode::Num7 | Keycode::Kp7 => Key::Char('7'),
            Keycode::Num8 | Keycode::Kp8 => Key::Char('8'),
            Keycode::Num9 | Keycode::Kp9 => Key::Char('9'),
            _ => return None,
        };

        Some(InputEvent::Key(KeyEvent { key, state }))
    }

    fn encoder_event(scroll_delta: Point, direction: MouseWheelDirection) -> Option<InputEvent> {
        let axis = if scroll_delta.y != 0 {
            scroll_delta.y
        } else {
            scroll_delta.x
        };
        if axis == 0 {
            return None;
        }

        let delta = match direction {
            MouseWheelDirection::Flipped => -axis,
            _ => axis,
        };
        Some(InputEvent::Encoder(EncoderEvent { delta }))
    }
}

impl Platform for SimulatorPlatform {
    type Color = Rgb565;
    type Error = Infallible;

    async fn next_event(&mut self) -> Option<InputEvent> {
        loop {
            // Deliver a Down/Up buffered from a previous batch first.
            if let Some(ev) = self.pending.take() {
                return Some(ev);
            }

            // Drain the whole SDL batch, coalescing consecutive mouse moves
            // into just the latest position. Returning on the first move
            // (the old behaviour) left the rest queued, so during a fast
            // drag the rendered position trailed the cursor and the backlog
            // grew — visible as lag. A discrete Down/Up that lands in the
            // same batch as moves is buffered in `self.pending` and the
            // pending move is flushed first, so no event is lost.
            let mut latest_move: Option<Point> = None;
            for sim_event in self.window.events() {
                match sim_event {
                    SimulatorEvent::Quit => return None,
                    SimulatorEvent::MouseButtonDown {
                        mouse_btn: MouseButton::Left,
                        point,
                    } => {
                        self.mouse_down = true;
                        let down = InputEvent::Touch(TouchEvent {
                            phase: TouchPhase::Down,
                            point,
                        });
                        if let Some(p) = latest_move.take() {
                            self.pending = Some(down);
                            return Some(InputEvent::Touch(TouchEvent {
                                phase: TouchPhase::Moved,
                                point: p,
                            }));
                        }
                        return Some(down);
                    }
                    SimulatorEvent::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        point,
                    } => {
                        self.mouse_down = false;
                        let up = InputEvent::Touch(TouchEvent {
                            phase: TouchPhase::Up,
                            point,
                        });
                        if let Some(p) = latest_move.take() {
                            self.pending = Some(up);
                            return Some(InputEvent::Touch(TouchEvent {
                                phase: TouchPhase::Moved,
                                point: p,
                            }));
                        }
                        return Some(up);
                    }
                    SimulatorEvent::MouseMove { point } if self.mouse_down => {
                        latest_move = Some(point);
                    }
                    SimulatorEvent::KeyDown {
                        keycode, repeat, ..
                    } => {
                        if let Some(event) = Self::key_event(keycode, repeat, true) {
                            return Some(event);
                        }
                    }
                    SimulatorEvent::KeyUp {
                        keycode, repeat, ..
                    } => {
                        if let Some(event) = Self::key_event(keycode, repeat, false) {
                            return Some(event);
                        }
                    }
                    SimulatorEvent::MouseWheel {
                        scroll_delta,
                        direction,
                    } => {
                        if let Some(event) = Self::encoder_event(scroll_delta, direction) {
                            return Some(event);
                        }
                    }
                    _ => {}
                }
            }
            if let Some(point) = latest_move {
                return Some(InputEvent::Touch(TouchEvent {
                    phase: TouchPhase::Moved,
                    point,
                }));
            }
            Timer::after(Duration::from_millis(self.poll_ms)).await;
        }
    }

    async fn render_with<F>(&mut self, draw: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut dyn Renderer<Self::Color>) -> Result<(), RenderError>,
    {
        // Reset pixmap to opaque black so AA edges blend against a
        // defined background; the runtime will paint the theme bg first.
        self.pixmap.fill(SkColor::BLACK);
        {
            let mut renderer = TinySkiaRenderer {
                pixmap: &mut self.pixmap,
                clip_mask: None,
                clip_rect: None,
                clip_stack: Vec::new(),
            };
            let _ = draw(&mut renderer);
        }

        // RGBA8 → Rgb565 blit into the SDL framebuffer.
        let area = Rectangle::new(Point::zero(), self.size);
        let data = self.pixmap.data();
        let colors = (0..(self.size.width * self.size.height) as usize).map(|i| {
            let off = i * 4;
            rgba8_to_rgb565(data[off], data[off + 1], data[off + 2])
        });
        let _ = self.display.fill_contiguous(&area, colors);

        self.window.update(&self.display);
        Ok(())
    }

    async fn render_with_dirty<F>(
        &mut self,
        dirty: &DirtyRegion,
        draw: F,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut dyn Renderer<Self::Color>) -> Result<(), RenderError>,
    {
        self.pixmap.fill(SkColor::BLACK);
        {
            let mut renderer = TinySkiaRenderer {
                pixmap: &mut self.pixmap,
                clip_mask: None,
                clip_rect: None,
                clip_stack: Vec::new(),
            };
            let _ = draw(&mut renderer);
        }

        match dirty {
            DirtyRegion::None => {}
            DirtyRegion::Full => self.blit_full(),
            DirtyRegion::Rects(rects) => {
                if self.show_dirty {
                    self.overlay_dirty(rects);
                }
                for rect in rects {
                    self.blit_rect(*rect);
                }
            }
        }

        self.window.update(&self.display);
        Ok(())
    }

    fn viewport(&self) -> Size {
        self.size
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            supports_clip: true,
            supports_partial_flush: true,
            supports_semantic_input: false,
            prefers_full_redraw: false,
        }
    }
}

impl SimulatorPlatform {
    fn blit_full(&mut self) {
        let area = Rectangle::new(Point::zero(), self.size);
        let data = self.pixmap.data();
        let colors = (0..(self.size.width * self.size.height) as usize).map(|i| {
            let off = i * 4;
            rgba8_to_rgb565(data[off], data[off + 1], data[off + 2])
        });
        let _ = self.display.fill_contiguous(&area, colors);
    }

    fn blit_rect(&mut self, rect: Rectangle) {
        let x0 = rect.top_left.x.max(0) as u32;
        let y0 = rect.top_left.y.max(0) as u32;
        let x1 = (rect.top_left.x + rect.size.width as i32).clamp(0, self.size.width as i32) as u32;
        let y1 =
            (rect.top_left.y + rect.size.height as i32).clamp(0, self.size.height as i32) as u32;
        if x1 <= x0 || y1 <= y0 {
            return;
        }

        let width = x1 - x0;
        let height = y1 - y0;
        let area = Rectangle::new(Point::new(x0 as i32, y0 as i32), Size::new(width, height));
        let stride = self.size.width as usize * 4;
        let data = self.pixmap.data();
        let mut colors = Vec::with_capacity((width * height) as usize);
        for y in y0..y1 {
            let row = y as usize * stride;
            for x in x0..x1 {
                let off = row + x as usize * 4;
                colors.push(rgba8_to_rgb565(data[off], data[off + 1], data[off + 2]));
            }
        }
        let _ = self.display.fill_contiguous(&area, colors.into_iter());
    }

    fn overlay_dirty(&mut self, rects: &[Rectangle]) {
        let mut renderer = TinySkiaRenderer {
            pixmap: &mut self.pixmap,
            clip_mask: None,
            clip_rect: None,
            clip_stack: Vec::new(),
        };
        for rect in rects {
            let _ = renderer.stroke_rect(*rect, Rgb565::MAGENTA);
        }
    }
}

struct TinySkiaRenderer<'p> {
    pixmap: &'p mut Pixmap,
    clip_mask: Option<Mask>,
    clip_rect: Option<Rectangle>,
    clip_stack: Vec<(Option<Mask>, Option<Rectangle>)>,
}

impl<'p> Renderer<Rgb565> for TinySkiaRenderer<'p> {
    fn fill_rect(&mut self, rect: Rectangle, color: Rgb565) -> Result<(), RenderError> {
        let Some(sk_rect) = SkRect::from_xywh(
            rect.top_left.x as f32,
            rect.top_left.y as f32,
            rect.size.width as f32,
            rect.size.height as f32,
        ) else {
            return Ok(());
        };
        let mut paint = Paint::default();
        paint.set_color(rgb565_to_skia(color));
        // Snap to pixel grid; blocks of solid color don't benefit from AA.
        paint.anti_alias = false;
        self.pixmap.fill_rect(
            sk_rect,
            &paint,
            Transform::identity(),
            self.clip_mask.as_ref(),
        );
        Ok(())
    }

    fn stroke_rect(&mut self, rect: Rectangle, color: Rgb565) -> Result<(), RenderError> {
        let Some(sk_rect) = SkRect::from_xywh(
            rect.top_left.x as f32 + 0.5,
            rect.top_left.y as f32 + 0.5,
            rect.size.width.saturating_sub(1) as f32,
            rect.size.height.saturating_sub(1) as f32,
        ) else {
            return Ok(());
        };
        let path = PathBuilder::from_rect(sk_rect);
        let mut paint = Paint::default();
        paint.set_color(rgb565_to_skia(color));
        paint.anti_alias = false;
        let mut stroke = Stroke::default();
        stroke.width = 1.0;
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            Transform::identity(),
            self.clip_mask.as_ref(),
        );
        Ok(())
    }

    fn fill_circle(
        &mut self,
        center: Point,
        radius: u32,
        color: Rgb565,
    ) -> Result<(), RenderError> {
        if radius == 0 {
            return Ok(());
        }
        let mut pb = PathBuilder::new();
        pb.push_circle(center.x as f32 + 0.5, center.y as f32 + 0.5, radius as f32);
        let Some(path) = pb.finish() else {
            return Ok(());
        };
        let mut paint = Paint::default();
        paint.set_color(rgb565_to_skia(color));
        paint.anti_alias = true;
        self.pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            self.clip_mask.as_ref(),
        );
        Ok(())
    }

    fn stroke_line(
        &mut self,
        start: Point,
        end: Point,
        color: Rgb565,
        width: u32,
    ) -> Result<(), RenderError> {
        if width == 0 {
            return Ok(());
        }
        let mut pb = PathBuilder::new();
        pb.move_to(start.x as f32 + 0.5, start.y as f32 + 0.5);
        pb.line_to(end.x as f32 + 0.5, end.y as f32 + 0.5);
        let Some(path) = pb.finish() else {
            return Ok(());
        };
        let mut paint = Paint::default();
        paint.set_color(rgb565_to_skia(color));
        paint.anti_alias = true;
        let mut stroke = Stroke::default();
        stroke.width = width as f32;
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            Transform::identity(),
            self.clip_mask.as_ref(),
        );
        Ok(())
    }

    fn stroke_arc(
        &mut self,
        center: Point,
        radius: u32,
        start_deg: i32,
        sweep_deg: i32,
        width: u32,
        color: Rgb565,
    ) -> Result<(), RenderError> {
        if radius == 0 || width == 0 || sweep_deg == 0 {
            return Ok(());
        }
        let Some(path) = arc_path(center, radius, start_deg, sweep_deg, false) else {
            return Ok(());
        };
        let mut paint = Paint::default();
        paint.set_color(rgb565_to_skia(color));
        paint.anti_alias = true;
        let mut stroke = Stroke::default();
        stroke.width = width as f32;
        stroke.line_cap = tiny_skia::LineCap::Round;
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            Transform::identity(),
            self.clip_mask.as_ref(),
        );
        Ok(())
    }

    fn fill_arc(
        &mut self,
        center: Point,
        radius: u32,
        start_deg: i32,
        sweep_deg: i32,
        color: Rgb565,
    ) -> Result<(), RenderError> {
        if radius == 0 || sweep_deg == 0 {
            return Ok(());
        }
        let Some(path) = arc_path(center, radius, start_deg, sweep_deg, true) else {
            return Ok(());
        };
        let mut paint = Paint::default();
        paint.set_color(rgb565_to_skia(color));
        paint.anti_alias = true;
        self.pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            self.clip_mask.as_ref(),
        );
        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font: &MonoFont<'_>,
        color: Rgb565,
        alignment: Alignment,
    ) -> Result<(), RenderError> {
        let style = MonoTextStyle::new(font, color);
        let mut adapter = PixmapDrawTarget {
            pixmap: &mut *self.pixmap,
            clip: self.clip_rect,
        };
        Text::with_alignment(text, position, style, alignment)
            .draw(&mut adapter)
            .map(|_| ())
            .map_err(|_| RenderError)
    }

    fn draw_image(
        &mut self,
        top_left: Point,
        size: Size,
        pixels: &[Rgb565],
    ) -> Result<(), RenderError> {
        let w = size.width as i32;
        if w == 0 {
            return Ok(());
        }
        let pw = self.pixmap.width() as i32;
        let ph = self.pixmap.height() as i32;
        let stride = self.pixmap.width() as usize * 4;
        let (cx1, cy1, cx2, cy2) = match self.clip_rect {
            Some(r) => (
                r.top_left.x,
                r.top_left.y,
                r.top_left.x + r.size.width as i32,
                r.top_left.y + r.size.height as i32,
            ),
            None => (0, 0, pw, ph),
        };
        let data = self.pixmap.data_mut();
        for (i, color) in pixels.iter().enumerate() {
            let x = top_left.x + (i as i32 % w);
            let y = top_left.y + (i as i32 / w);
            if x < cx1 || y < cy1 || x >= cx2 || y >= cy2 {
                continue;
            }
            if x < 0 || y < 0 || x >= pw || y >= ph {
                continue;
            }
            let off = y as usize * stride + x as usize * 4;
            let (r, g, b) = rgb565_components(*color);
            data[off] = r;
            data[off + 1] = g;
            data[off + 2] = b;
            data[off + 3] = 255;
        }
        Ok(())
    }

    fn push_clip(&mut self, rect: Rectangle) {
        let new_rect = match self.clip_rect {
            Some(existing) => intersect(existing, rect),
            None => rect,
        };
        let new_mask = if new_rect.size.width == 0 || new_rect.size.height == 0 {
            // Empty clip — build a fully-black mask (nothing draws).
            Some(Mask::new(self.pixmap.width(), self.pixmap.height()).expect("mask"))
        } else {
            build_rect_mask(self.pixmap.width(), self.pixmap.height(), new_rect)
        };
        let prev_mask = core::mem::replace(&mut self.clip_mask, new_mask);
        let prev_rect = self.clip_rect.replace(new_rect);
        self.clip_stack.push((prev_mask, prev_rect));
    }

    fn pop_clip(&mut self) {
        if let Some((mask, rect)) = self.clip_stack.pop() {
            self.clip_mask = mask;
            self.clip_rect = rect;
        }
    }
}

fn intersect(a: Rectangle, b: Rectangle) -> Rectangle {
    let ax2 = a.top_left.x + a.size.width as i32;
    let ay2 = a.top_left.y + a.size.height as i32;
    let bx2 = b.top_left.x + b.size.width as i32;
    let by2 = b.top_left.y + b.size.height as i32;
    let x1 = a.top_left.x.max(b.top_left.x);
    let y1 = a.top_left.y.max(b.top_left.y);
    let x2 = ax2.min(bx2);
    let y2 = ay2.min(by2);
    if x2 <= x1 || y2 <= y1 {
        Rectangle::new(Point::new(x1, y1), Size::zero())
    } else {
        Rectangle::new(
            Point::new(x1, y1),
            Size::new((x2 - x1) as u32, (y2 - y1) as u32),
        )
    }
}

/// Build a tiny-skia path for a circular arc centered at `center`.
///
/// Sweeps `sweep_deg` degrees from `start_deg` (0° points right,
/// positive sweep is counter-clockwise on screen). When `pie` is true
/// the path is closed through the center to form a fillable sector;
/// otherwise it is an open polyline suitable for stroking.
fn arc_path(
    center: Point,
    radius: u32,
    start_deg: i32,
    sweep_deg: i32,
    pie: bool,
) -> Option<tiny_skia::Path> {
    let total = sweep_deg.unsigned_abs().min(360);
    let step: f32 = if sweep_deg >= 0 { 1.0 } else { -1.0 };
    let r = radius as f32;
    let cx = center.x as f32 + 0.5;
    let cy = center.y as f32 + 0.5;

    let point_at = |deg: f32| -> (f32, f32) {
        let rad = deg * core::f32::consts::PI / 180.0;
        // Screen y grows downward, so negate the sine.
        (cx + rad.cos() * r, cy - rad.sin() * r)
    };

    let mut pb = PathBuilder::new();
    if pie {
        pb.move_to(cx, cy);
        let (x0, y0) = point_at(start_deg as f32);
        pb.line_to(x0, y0);
    } else {
        let (x0, y0) = point_at(start_deg as f32);
        pb.move_to(x0, y0);
    }
    for i in 1..=total {
        let (x, y) = point_at(start_deg as f32 + i as f32 * step);
        pb.line_to(x, y);
    }
    if pie {
        pb.close();
    }
    pb.finish()
}

fn build_rect_mask(pixmap_w: u32, pixmap_h: u32, rect: Rectangle) -> Option<Mask> {
    let mut mask = Mask::new(pixmap_w, pixmap_h)?;
    let sk_rect = SkRect::from_xywh(
        rect.top_left.x as f32,
        rect.top_left.y as f32,
        rect.size.width as f32,
        rect.size.height as f32,
    )?;
    let path = PathBuilder::from_rect(sk_rect);
    mask.fill_path(&path, FillRule::Winding, false, Transform::identity());
    Some(mask)
}

/// `DrawTarget<Color = Rgb565>` adapter so embedded-graphics text and
/// other built-in primitives can write directly into the tiny-skia
/// pixmap. Used only for text in this backend; shapes go through
/// `TinySkiaRenderer` for AA.
struct PixmapDrawTarget<'p> {
    pixmap: &'p mut Pixmap,
    clip: Option<Rectangle>,
}

impl<'p> OriginDimensions for PixmapDrawTarget<'p> {
    fn size(&self) -> Size {
        Size::new(self.pixmap.width(), self.pixmap.height())
    }
}

impl<'p> DrawTarget for PixmapDrawTarget<'p> {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let w = self.pixmap.width() as i32;
        let h = self.pixmap.height() as i32;
        let stride = self.pixmap.width() as usize * 4;
        let (cx1, cy1, cx2, cy2) = match self.clip {
            Some(r) => (
                r.top_left.x,
                r.top_left.y,
                r.top_left.x + r.size.width as i32,
                r.top_left.y + r.size.height as i32,
            ),
            None => (0, 0, w, h),
        };
        let data = self.pixmap.data_mut();
        for Pixel(p, c) in pixels {
            if p.x < cx1 || p.y < cy1 || p.x >= cx2 || p.y >= cy2 {
                continue;
            }
            if p.x < 0 || p.y < 0 || p.x >= w || p.y >= h {
                continue;
            }
            let off = p.y as usize * stride + p.x as usize * 4;
            let (r, g, b) = rgb565_components(c);
            data[off] = r;
            data[off + 1] = g;
            data[off + 2] = b;
            data[off + 3] = 255;
        }
        Ok(())
    }
}

fn rgb565_components(c: Rgb565) -> (u8, u8, u8) {
    // Upscale 5/6/5-bit channels to 8 by replicating high bits into the lows.
    let r5 = c.r();
    let g6 = c.g();
    let b5 = c.b();
    let r = (r5 << 3) | (r5 >> 2);
    let g = (g6 << 2) | (g6 >> 4);
    let b = (b5 << 3) | (b5 >> 2);
    (r, g, b)
}

fn rgb565_to_skia(c: Rgb565) -> SkColor {
    let (r, g, b) = rgb565_components(c);
    SkColor::from_rgba8(r, g, b, 255)
}

fn rgba8_to_rgb565(r: u8, g: u8, b: u8) -> Rgb565 {
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}
