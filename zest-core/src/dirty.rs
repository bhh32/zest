//! Dirty-region tracking and platform rendering capabilities.

use alloc::{vec, vec::Vec};
use embedded_graphics::primitives::Rectangle;

/// Dirty regions to redraw on the next frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirtyRegion {
    /// Nothing changed; drawing can be skipped.
    None,
    /// The whole surface should be redrawn.
    Full,
    /// Redraw the listed rectangles.
    Rects(Vec<Rectangle>),
}

impl DirtyRegion {
    /// Nothing is dirty.
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    /// The whole surface is dirty.
    #[must_use]
    pub const fn full() -> Self {
        Self::Full
    }

    /// A single dirty rectangle.
    #[must_use]
    pub fn rect(rect: Rectangle) -> Self {
        Self::Rects(vec![rect])
    }

    /// True if no redraw is needed.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Borrow the dirty rectangles when the region is rectangular.
    #[must_use]
    pub fn rects(&self) -> Option<&[Rectangle]> {
        match self {
            Self::Rects(rects) => Some(rects.as_slice()),
            _ => None,
        }
    }

    /// Add a rectangle to the dirty set.
    pub fn add_rect(&mut self, rect: Rectangle) {
        match self {
            Self::None => *self = Self::Rects(vec![rect]),
            Self::Full => {}
            Self::Rects(rects) => rects.push(rect),
        }
    }
}

/// Renderer/backend capabilities relevant to the runtime.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PlatformCapabilities {
    /// The renderer can clip draw calls to a sub-rectangle.
    pub supports_clip: bool,
    /// The backend can flush only dirty rectangles efficiently.
    pub supports_partial_flush: bool,
    /// The backend accepts semantic input actions directly.
    pub supports_semantic_input: bool,
    /// The backend is happier redrawing the whole frame.
    pub prefers_full_redraw: bool,
}
