//! Bitmap typography scale — four monospace font sizes the theme
//! exposes by role, so widgets can ask for `theme.typography.display`
//! or `theme.typography.body` without picking a `MonoFont` constant
//! directly.

use embedded_graphics::mono_font::MonoFont;

/// Typography scale. `display` covers hero text on larger panels,
/// while `heading` / `body` / `caption` handle the regular UI scale.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Typography<'a> {
    /// Hero text, such as a wall clock face.
    pub display: &'a MonoFont<'a>,
    /// Section headings and other large in-flow UI text.
    pub heading: &'a MonoFont<'a>,
    /// Body / label text. The default font widgets use.
    pub body: &'a MonoFont<'a>,
    /// Small captions, error strings, hints.
    pub caption: &'a MonoFont<'a>,
}

impl<'a> Typography<'a> {
    /// Construct a typography scale.
    #[must_use]
    pub const fn new(
        display: &'a MonoFont<'a>,
        heading: &'a MonoFont<'a>,
        body: &'a MonoFont<'a>,
        caption: &'a MonoFont<'a>,
    ) -> Self {
        Self {
            display,
            heading,
            body,
            caption,
        }
    }
}
