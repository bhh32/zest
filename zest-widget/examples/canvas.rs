//! Canvas: an owned [`CanvasBuffer`] the host draws into, blitted to the
//! screen by the passive [`Canvas`] widget.
//!
//! Because widgets are rebuilt every frame, the mutable pixel state lives on
//! the host as a [`CanvasBuffer`] field. The host mutates it in `update`
//! (here a button cycles through a few drawings) and lends it to the `Canvas`
//! widget by reference in `view`.

extern crate alloc;
use zest::prelude::*;
use zest::zest_theme::theme::dark;

const CANVAS_W: u32 = 220;
const CANVAS_H: u32 = 160;

#[derive(Clone)]
enum Msg {
    Next,
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    buffer: CanvasBuffer<Rgb565>,
    pattern: u8,
}

impl Screen {
    fn new() -> Self {
        let mut s = Self {
            theme: convert_theme(&dark::THEME),
            buffer: CanvasBuffer::new(CANVAS_W, CANVAS_H, Rgb565::new(0, 0, 0)),
            pattern: 0,
        };
        s.redraw();
        s
    }

    /// Paint the current pattern into the owned buffer.
    fn redraw(&mut self) {
        let p = &self.theme.palette;
        self.buffer.clear(p.neutral_10);
        match self.pattern % 3 {
            0 => {
                // Concentric rectangle frames.
                let mut i = 0i32;
                while i < (CANVAS_W.min(CANVAS_H) as i32) / 2 {
                    let color = if (i / 8) % 2 == 0 { p.accent_blue } else { p.accent_green };
                    self.buffer.fill_rect(
                        i,
                        i,
                        CANVAS_W.saturating_sub(2 * i as u32),
                        CANVAS_H.saturating_sub(2 * i as u32),
                        color,
                    );
                    i += 8;
                }
            }
            1 => {
                // A starburst of lines from the center.
                let cx = CANVAS_W as i32 / 2;
                let cy = CANVAS_H as i32 / 2;
                let mut x = 0i32;
                while x < CANVAS_W as i32 {
                    self.buffer.line(cx, cy, x, 0, p.accent_red);
                    self.buffer.line(cx, cy, x, CANVAS_H as i32 - 1, p.accent_yellow);
                    x += 12;
                }
            }
            _ => {
                // A diagonal RGB565 gradient drawn pixel-by-pixel.
                let mut y = 0i32;
                while y < CANVAS_H as i32 {
                    let mut x = 0i32;
                    while x < CANVAS_W as i32 {
                        let r = (x as u32 * 31 / CANVAS_W) as u8;
                        let g = (y as u32 * 63 / CANVAS_H) as u8;
                        self.buffer.set_pixel(x, y, Rgb565::new(r, g, 12));
                        x += 1;
                    }
                    y += 1;
                }
            }
        }
    }
}

impl ScreenView<Rgb565, Msg> for Screen {
    fn name(&self) -> &'static str {
        "Canvas"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        let heading = Text::new("Canvas drawing")
            .align_x(Horizontal::Center)
            .font(self.theme.typography.heading)
            .color(self.theme.background.on_base);

        Column::new()
            .spacing(8)
            .push(heading)
            .push(horizontal_divider())
            .push(Canvas::new(&self.buffer))
            .push(
                Button::new("Next pattern")
                    .class(ButtonClass::Suggested)
                    .on_press(Msg::Next),
            )
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Msg;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Msg>) {
        (Self { screen: Screen::new() }, Task::none())
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Next => {
                self.screen.pattern = self.screen.pattern.wrapping_add(1);
                self.screen.redraw();
            }
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Canvas").await;
}
