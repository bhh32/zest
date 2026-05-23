//! Standard widget library for `zest`.

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate alloc;

/// Concrete widgets.
pub mod widget;

pub use widget::Image;
pub use widget::Widget;
pub use widget::arc::Arc;
pub use widget::button::Button;
pub use widget::calendar::{Calendar, CalendarEvent};
pub use widget::chart::Chart;
pub use widget::checkbox::Checkbox;
pub use widget::column::Column;
pub use widget::container::Container;
pub use widget::divider::{Divider, horizontal_divider, vertical_divider};
pub use widget::element::{Element, IntoElement};
pub use widget::grid::Grid;
pub use widget::keyboard::{KeyAction, Keyboard, Layout};
pub use widget::led::LED;
pub use widget::line::Line;
pub use widget::progress_bar::ProgressBar;
pub use widget::radio::RadioButton;
pub use widget::row::Row;
pub use widget::scale::{Scale, ScaleMode};
pub use widget::scrollable::Scrollable;
pub use widget::slider::Slider;
pub use widget::space::{Space, horizontal_space, vertical_space};
pub use widget::spin_button::{
    SpinButton, SpinOrientation, horizontal_spin_button, vertical_spin_button,
};
pub use widget::spinner::Spinner;
pub use widget::stack::Stack;
pub use widget::switch::Switch;
pub use widget::tab_bar::{Tab, TabBar};
pub use widget::text::Text;
pub use widget::weather_icons::{WeatherCondition, WeatherIcon};
pub use zest_core::scroll::{
    ScrollDirection, ScrollMsg, ScrollState, ScrollbarMode, SnapMode, tick_task,
};
