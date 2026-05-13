//! Touch-driven UI widgets for `embedded-graphics` displays.
//!
//! This crate provides interactive widgets — buttons, on-screen keyboards —
//! for small embedded displays where the user input mechanism is a touch
//! controller (resistive or capacitive). Touches arrive as `Point` values;
//! widgets handle hit-testing, state, and rendering.
//!
//! # Design
//!
//! Widgets are *generic over color* (`<C: PixelColor>`) so the same code
//! works on RGB565 TFTs, monochrome OLEDs, or e-paper. Each widget exposes a
//! `Theme` struct that bundles its colors; callers construct one and pass it
//! in.
//!
//! Widgets are state-owning: they hold their own state (input buffer, shift
//! key state, pressed-key state) across frames. The caller polls them with
//! `handle_touch(point)` for input and `draw(display)` for rendering.
//! `handle_touch` returns an event enum; the widget never invokes callbacks.
//!
//! # Example
//!
//! ```no_run
//! use embedded_widgets::{Button, ButtonTheme};
//! use embedded_graphics::{
//!     pixelcolor::Rgb565,
//!     prelude::*,
//!     primitives::Rectangle,
//! };
//!
//! let theme = ButtonTheme::<Rgb565>::default_dark();
//! let button = Button::new(
//!     Rectangle::new(Point::new(10, 10), Size::new(80, 30)),
//!     "Save",
//!     theme,
//! );
//! // button.draw(&mut display)?;
//! // if button.hit_test(touch_point) { ... }
//! ```
//!
//! # Modules
//!
//! - [`button`] — tappable rectangular button with a label
//! - [`keyboard`] — on-screen QWERTY keyboard with alpha/numeric layouts

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod button;
pub mod core;
pub mod column;
pub mod container;
pub mod element;
pub mod grid;
pub mod keyboard;
pub mod renderer;
pub mod row;
pub mod theme;
pub mod widget;

// Re-exports
pub use button::Button;
pub use element::{Element, IntoElement};
pub use keyboard::{Keyboard, KeyboardEvent};
pub use renderer::Renderer;
pub use theme::{ButtonStyle, LabelStyle, Theme};
pub use widget::Widget;
