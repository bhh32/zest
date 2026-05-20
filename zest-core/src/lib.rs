//! Application contract, async runtime, and Widget trait for `zest`.

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod application;
pub mod event;
pub mod layout;
pub mod platform;
pub mod renderer;
pub mod runtime;
pub mod screen;
pub mod time;
pub mod widget;

pub use application::{Application, Recipe, Subscription, Task};
pub use event::{InputEvent, TickResult, TouchEvent, TouchPhase};
pub use layout::{Constraints, Horizontal, Length, UNBOUNDED, Vertical};
pub use platform::Platform;
pub use renderer::{DrawTargetRenderer, RenderError, Renderer};
pub use runtime::Runtime;
pub use screen::ScreenView;
pub use widget::{Element, IntoElement, Widget};
