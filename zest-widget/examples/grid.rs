//! Four-function calculator on a 4x5 grid. Demonstrates the `Grid`
//! widget plus `ButtonClass` semantic variants (numerics are standard,
//! operators are suggested, `=` is success, `C` is destructive,
//! backspace is a warning).

extern crate alloc;
use alloc::{format, string::String};
use zest::prelude::*;
use zest::zest_theme::theme::dark;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => {
                if b == 0.0 {
                    f64::NAN
                } else {
                    a / b
                }
            }
        }
    }
}

#[derive(Clone)]
enum Key {
    Digit(char),
    Dot,
    Sign,
    Percent,
    Op(Op),
    Equals,
    Clear,
    Backspace,
}

struct Calc {
    display: String,
    acc: Option<f64>,
    pending: Option<Op>,
    // `display` currently shows the result of `=`. Next digit replaces it.
    just_evaluated: bool,
    // True iff the next digit should overwrite the display (fresh operand).
    fresh: bool,
}

impl Calc {
    fn new() -> Self {
        Self {
            display: String::from("0"),
            acc: None,
            pending: None,
            just_evaluated: false,
            fresh: true,
        }
    }

    fn current(&self) -> f64 {
        self.display.parse::<f64>().unwrap_or(0.0)
    }

    fn show(&mut self, v: f64) {
        if v.is_nan() {
            self.display = String::from("err");
        } else if v == v.trunc() && v.abs() < 1e12 {
            self.display = format!("{}", v as i64);
        } else {
            // 8 significant digits; trim trailing zeros / dot.
            let s = format!("{v:.8}");
            let trimmed = s.trim_end_matches('0').trim_end_matches('.').to_string();
            self.display = if trimmed.is_empty() {
                String::from("0")
            } else {
                trimmed
            };
        }
    }

    fn push_digit(&mut self, c: char) {
        if self.just_evaluated || self.fresh {
            self.display.clear();
            self.just_evaluated = false;
            self.fresh = false;
        }
        if self.display.len() >= 12 {
            return;
        }
        if self.display == "0" {
            self.display.clear();
        }
        self.display.push(c);
    }

    fn push_dot(&mut self) {
        if self.just_evaluated || self.fresh {
            self.display = String::from("0");
            self.just_evaluated = false;
            self.fresh = false;
        }
        if !self.display.contains('.') && self.display.len() < 12 {
            self.display.push('.');
        }
    }

    fn op(&mut self, op: Op) {
        let cur = self.current();
        if let (Some(acc), Some(pending), false) = (self.acc, self.pending, self.fresh) {
            let r = pending.apply(acc, cur);
            self.show(r);
            self.acc = Some(r);
        } else {
            self.acc = Some(cur);
        }
        self.pending = Some(op);
        self.fresh = true;
        self.just_evaluated = false;
    }

    fn equals(&mut self) {
        if let (Some(acc), Some(pending)) = (self.acc, self.pending) {
            let r = pending.apply(acc, self.current());
            self.show(r);
            self.acc = Some(r);
            self.pending = None;
            self.just_evaluated = true;
            self.fresh = true;
        }
    }

    fn clear(&mut self) {
        *self = Calc::new();
    }

    fn backspace(&mut self) {
        if self.just_evaluated || self.fresh {
            return;
        }
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = String::from("0");
        }
    }

    fn toggle_sign(&mut self) {
        if self.display == "0" {
            return;
        }
        if let Some(stripped) = self.display.strip_prefix('-') {
            self.display = stripped.into();
        } else {
            self.display.insert(0, '-');
        }
    }

    fn percent(&mut self) {
        let v = self.current() / 100.0;
        self.show(v);
        self.fresh = true;
    }
}

struct Screen {
    theme: Theme<'static, Rgb565>,
    calc: Calc,
}

impl ScreenView<Rgb565, Key> for Screen {
    fn name(&self) -> &'static str {
        "Calculator"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Key> {
        // Row 0: C, backspace, %, ÷
        // Rows 1–3: digits with ×, −, +
        // Row 4: ±, 0, ., =
        let grid = Grid::new(4, 5)
            .spacing(4)
            .push(
                Button::new("C")
                    .on_press(Key::Clear)
                    .class(ButtonClass::Destructive),
            )
            .push(
                Button::new("<-")
                    .on_press(Key::Backspace)
                    .class(ButtonClass::Warning),
            )
            .push(Button::new("%").on_press(Key::Percent))
            .push(
                Button::new("/")
                    .on_press(Key::Op(Op::Div))
                    .class(ButtonClass::Suggested),
            )
            .push(Button::new("7").on_press(Key::Digit('7')))
            .push(Button::new("8").on_press(Key::Digit('8')))
            .push(Button::new("9").on_press(Key::Digit('9')))
            .push(
                Button::new("*")
                    .on_press(Key::Op(Op::Mul))
                    .class(ButtonClass::Suggested),
            )
            .push(Button::new("4").on_press(Key::Digit('4')))
            .push(Button::new("5").on_press(Key::Digit('5')))
            .push(Button::new("6").on_press(Key::Digit('6')))
            .push(
                Button::new("-")
                    .on_press(Key::Op(Op::Sub))
                    .class(ButtonClass::Suggested),
            )
            .push(Button::new("1").on_press(Key::Digit('1')))
            .push(Button::new("2").on_press(Key::Digit('2')))
            .push(Button::new("3").on_press(Key::Digit('3')))
            .push(
                Button::new("+")
                    .on_press(Key::Op(Op::Add))
                    .class(ButtonClass::Suggested),
            )
            .push(Button::new("+/-").on_press(Key::Sign))
            .push(Button::new("0").on_press(Key::Digit('0')))
            .push(Button::new(".").on_press(Key::Dot))
            .push(
                Button::new("=")
                    .on_press(Key::Equals)
                    .class(ButtonClass::Success),
            );

        Column::new()
            .spacing(2)
            .push(
                Container::new().height(Length::Fixed(36)).padding(4).child(
                    Text::new(self.calc.display.clone())
                        .align_x(Horizontal::Right)
                        .align_y(Vertical::Center)
                        .font(self.theme.typography.heading)
                        .color(self.theme.background.on_base),
                ),
            )
            .push(grid)
            .into_element()
    }
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Message = Key;
    type Color = Rgb565;
    type Screen = Screen;

    fn init() -> (Self, Task<Key>) {
        (
            Self {
                screen: Screen {
                    theme: convert_theme(&dark::THEME),
                    calc: Calc::new(),
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, k: Key) -> Task<Key> {
        let c = &mut self.screen.calc;
        match k {
            Key::Digit(d) => c.push_digit(d),
            Key::Dot => c.push_dot(),
            Key::Sign => c.toggle_sign(),
            Key::Percent => c.percent(),
            Key::Op(op) => c.op(op),
            Key::Equals => c.equals(),
            Key::Clear => c.clear(),
            Key::Backspace => c.backspace(),
        }
        Task::none()
    }

    fn view(&self) -> &Screen {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Calculator").await;
}
