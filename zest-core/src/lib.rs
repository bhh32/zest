//! Application contract, async runtime, and Widget trait for `zest`.

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod application;
pub mod dirty;
pub mod event;
pub mod focus;
pub mod layout;
pub mod platform;
pub mod renderer;
pub mod runtime;
pub mod screen;
pub mod scroll;
pub mod time;
pub mod widget;

pub use application::{Application, Recipe, Subscription, Task};
pub use dirty::{DirtyRegion, PlatformCapabilities};
pub use event::{
    ButtonState, EncoderEvent, InputEvent, Key, KeyEvent, TickResult, TouchEvent, TouchPhase,
    UiAction,
};
pub use focus::{FocusDirection, FocusState, WidgetId};
pub use layout::{Constraints, Horizontal, Length, UNBOUNDED, Vertical};
pub use platform::Platform;
pub use renderer::{DrawTargetRenderer, RenderError, Renderer, arc_sin_cos};
pub use runtime::Runtime;
pub use screen::ScreenView;
pub use scroll::{
    GesturePhase, ScrollDirection, ScrollMsg, ScrollState, ScrollbarMode, SnapMode, tick_task,
};
pub use widget::{Element, IntoElement, Widget};
