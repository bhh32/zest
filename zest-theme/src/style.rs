//! Catalog/Status/Appearance pattern, modeled on libcosmic but trimmed
//! for embedded touch:
//!
//! - **No hover state.** Touch screens have no pointer-over phase.
//! - **No animations / transitions.** Allocate-and-draw budget on an
//!   ESP32 doesn't have room for them.
//!
//! A catalog is a function on the [`Theme`](crate::Theme): given a
//! widget *class* (variant — `Standard`, `Suggested`, `Destructive`,
//! …) and a *status* (`Active`, `Focused`, `Pressed`, `Disabled`), return a
//! resolved [`ButtonAppearance`] the widget can paint directly.
//! Widgets never reach into the theme's `Component`/`Container` fields;
//! they call the catalog method.

use embedded_graphics::pixelcolor::PixelColor;

/// Interaction state of a stateful widget. Single shared enum across
/// all interactive widgets — keeps the catalog dispatch table small
/// and the embedded code path predictable.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Resting state — not pressed, not disabled.
    Active,
    /// Focused through non-touch traversal.
    Focused,
    /// Currently being touched.
    Pressed,
    /// No-op state (e.g. button has no `on_press`).
    Disabled,
}

/// Semantic variant of a button. The catalog maps a class to one of
/// the theme's interactive components (button / accent / destructive
/// / …) without leaking those field names to widget code.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ButtonClass {
    /// Default filled button. Maps to `theme.button`.
    #[default]
    Standard,
    /// Primary call-to-action. Maps to `theme.accent`.
    Suggested,
    /// Destructive action (delete, cancel-with-loss). Maps to `theme.destructive`.
    Destructive,
    /// Positive confirmation. Maps to `theme.success`.
    Success,
    /// Cautionary action. Maps to `theme.warning`.
    Warning,
    /// No fill, no border — text-only. Maps to `theme.text_button`.
    Text,
    /// Icon-only target. Maps to `theme.icon_button`.
    Icon,
}

/// Resolved button style — what the widget actually paints.
///
/// `Option`s encode "no fill" / "no border" so a `ButtonClass::Text`
/// can return `background = None, border = None` without us inventing
/// a sentinel transparent color.
#[derive(Copy, Clone, Debug)]
pub struct ButtonAppearance<C: PixelColor> {
    /// Fill color. `None` = transparent.
    pub background: Option<C>,
    /// Border color. `None` = no border drawn.
    pub border: Option<C>,
    /// Foreground color (label text).
    pub text: C,
}

/// Catalog for buttons. Implemented by [`crate::Theme`].
pub trait ButtonCatalog<C: PixelColor> {
    /// Resolve `(class, status)` to the colors the widget should paint.
    fn button(&self, class: ButtonClass, status: Status) -> ButtonAppearance<C>;
}
