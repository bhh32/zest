//! `zest` — retained-mode GUI framework for embedded touchscreen MCUs.
//!
//! Re-exports `zest-core`, `zest-theme`, `zest-widget`, and (with the
//! `simulator` feature) `zest-simulator`. Provides convenience runners
//! and a `time` module re-exported from `zest-core`.
//!
//! ```ignore
//! use zest::prelude::*;
//!
//! struct App { /* ... */ }
//!
//! impl Application for App {
//!     fn init() -> (Self, Task<Msg>) { /* ... */ }
//!     // ...
//! }
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: embassy_executor::Spawner) {
//!     zest::run::<App>("My App").await;
//! }
//! ```

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

#[cfg(any(feature = "simulator", feature = "net"))]
extern crate std;

pub use zest_core;
pub use zest_theme;
pub use zest_widget;

#[cfg(feature = "simulator")]
pub use zest_simulator;

/// Re-export of [`zest_core::time`]: built-in subscription helpers.
pub mod time {
    pub use zest_core::time::*;
}

/// Async network helpers for desktop development.
///
/// On the simulator the embassy executor doesn't host a tokio reactor,
/// so true async HTTP isn't available. This module spawns a `std::thread`
/// for the blocking call and yields between embassy timer ticks until
/// the worker finishes — entirely an implementation detail. The
/// signature stays `async fn`, matching what `embassy-net` + `reqwless`
/// will expose on hardware.
#[cfg(feature = "net")]
pub mod net;

/// Everything an application typically wants imported at the top.
///
/// Includes the framework primitives plus a curated set of
/// `embedded_graphics` types (`Rectangle`, `Point`, `Size`,
/// `Rgb565`, `Rgb888`, `BinaryColor`) so apps generally don't need
/// to depend on `embedded-graphics` directly.
pub mod prelude {
    pub use zest_core::{
        Application, Constraints, DrawTargetRenderer, Horizontal, InputEvent, Length, Platform,
        Recipe, RenderError, Renderer, Runtime, ScreenView, Subscription, Task, TickResult,
        TouchEvent, TouchPhase, Vertical,
    };
    pub use zest_theme::{
        ButtonAppearance, ButtonCatalog, ButtonClass, Component, CornerRadii, FONT_ZEST_MONO,
        FONT_ZEST_MONO_CAPTION, FONT_ZEST_MONO_DISPLAY, FONT_ZEST_MONO_HEADING, Palette, Spacing,
        Status, Theme, Typography, convert_theme,
    };
    pub use zest_widget::{
        Arc, Button, Calendar, CalendarEvent, Canvas, CanvasBuffer, Chart, Checkbox, Column,
        Container, Divider, Dropdown, EccLevel, Element, Grid, ImageButton, IntoElement, KeyAction,
        Keyboard, KeyboardMode, LED, Line, List, ListRow, Menu, MessageBox, ProgressBar, Qr,
        RadioButton, Roller, Row, Scale, ScaleMode, ScrollDirection, ScrollMsg, ScrollState,
        Scrollable, ScrollbarMode, Slider, SnapMode, Space, Span, SpanGroup, SpinButton,
        SpinOrientation, Spinner, Stack, Switch, Tab, TabBar, Table, TableRow, Text, TextArea,
        Tileview, Widget, Window, horizontal_divider, horizontal_space, horizontal_spin_button,
        tick_task, vertical_divider, vertical_space, vertical_spin_button,
    };

    pub use embedded_graphics::{
        pixelcolor::{BinaryColor, Rgb565, Rgb888},
        prelude::{Point, Size},
        primitives::Rectangle,
    };
}

/// Run application type `A` with the framework's default desktop
/// platform (the simulator).
///
/// libcosmic-shaped: the runtime calls `A::init()` to obtain the
/// initial app value and a startup task. For custom hardware,
/// construct a `Platform` yourself and call
/// `Runtime::<A>::new().run(platform).await` directly.
///
/// Calls `std::process::exit(0)` when the simulator window closes, since
/// embassy's std executor parks the main thread and would otherwise hang.
#[cfg(feature = "simulator")]
pub async fn run<A>(title: &str)
where
    A: zest_core::Application<Color = embedded_graphics::pixelcolor::Rgb565>,
{
    let platform = zest_simulator::SimulatorPlatform::new(title);
    zest_core::Runtime::<A>::new().run(platform).await;
    std::process::exit(0);
}
