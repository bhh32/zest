//! Theme: top-level visual config. Widgets read their style from here.

use embedded_graphics::{mono_font::MonoFont, pixelcolor::PixelColor};

/// Top-level theme. App owns one, passes to every `draw` call.
#[derive(Debug, Clone, Copy)]
pub struct Theme<'a, C: PixelColor> {
    /// Hint color for clearing the screen between renders.
    pub background: C,
    /// Default style for [`crate::Button`]
    pub button: ButtonStyle<'a, C>,
    /// Default style for labels.
    pub label: LabelStyle<'a, C>,
}

/// Visual style for a button.
#[derive(Debug, Clone, Copy)]
pub struct ButtonStyle<'a, C: PixelColor> {
    /// Fill in resting state.
    pub background: C,
    /// Fill while pressed.
    pub pressed_background: C,
    /// 1-pixel border.
    pub border: C,
    /// Label text color.
    pub label: C,
    /// Label font.
    pub font: &'a MonoFont<'a>,
}

/// Visual style for a label.
#[derive(Debug, Clone, Copy)]
pub struct LabelStyle<'a, C: PixelColor> {
    /// Text color.
    pub color: C,
    /// Font.
    pub font: &'a MonoFont<'a>,
}
