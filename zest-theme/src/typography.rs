//! Bitmap typography scale — three monospace font sizes the theme
//! exposes by role, so widgets can ask for `theme.typography.body`
//! without picking a `MonoFont` constant directly.

use embedded_graphics::mono_font::MonoFont;

/// Typography scale. Three roles is the right number for a 320×240
/// panel — a fourth would just split hairs at the same row height.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Typography<'a> {
    /// Section headings, large numerics (clock face, current temp).
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
        heading: &'a MonoFont<'a>,
        body: &'a MonoFont<'a>,
        caption: &'a MonoFont<'a>,
    ) -> Self {
        Self {
            heading,
            body,
            caption,
        }
    }
}
