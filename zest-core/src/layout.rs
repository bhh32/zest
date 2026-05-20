//! Layout protocol: [`Constraints`] flow down, [`Size`] flows up.
//!
//! `zest` uses a WPF/Flutter-style two-pass layout:
//!
//! 1. **Measure**: parent calls `child.measure(constraints)`, child returns
//!    its desired [`Size`] within `constraints`.
//! 2. **Arrange**: parent calls `child.arrange(rect)` with a concrete
//!    rectangle. Child stores `rect`, containers recursively arrange
//!    children.
//!
//! Each widget exposes a [`Length`] on each axis via
//! [`Widget::preferred_size`](crate::Widget::preferred_size). Containers
//! interpret these to allocate space:
//!
//! * [`Length::Fixed`] children get exactly that many pixels.
//! * [`Length::Shrink`] children get whatever they ask for via `measure`.
//! * [`Length::Fill`] / [`Length::FillPortion`] children share the
//!   remaining space proportionally (`Fill` is `FillPortion(1)`).

use embedded_graphics::prelude::*;

/// Sentinel for "unbounded" in a constraint's `max` field. Equal to
/// `u16::MAX as u32`; actual screen dimensions on every target this
/// framework supports fit comfortably under it.
pub const UNBOUNDED: u32 = u16::MAX as u32;

/// Min and max bounds a parent gives a child during the measure pass.
///
/// Children should return a size satisfying `min <= size <= max` on both
/// axes. The [`UNBOUNDED`] sentinel in a `max` field means "as much as you
/// want."
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Constraints {
    /// Minimum size the child must produce.
    pub min: Size,
    /// Maximum size the child may produce.
    pub max: Size,
}

impl Constraints {
    /// Constraints permitting any size up to `max`.
    #[must_use]
    pub const fn loose(max: Size) -> Self {
        Self {
            min: Size::zero(),
            max,
        }
    }

    /// Constraints requiring exactly `size`.
    #[must_use]
    pub const fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    /// Constraints permitting `min..=max`.
    #[must_use]
    pub const fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    /// Constraints from zero up to [`UNBOUNDED`] on both axes.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            min: Size::zero(),
            max: Size::new(UNBOUNDED, UNBOUNDED),
        }
    }

    /// Clamp `size` to fall within the constraints on both axes.
    #[must_use]
    pub fn clamp(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min.width, self.max.width),
            size.height.clamp(self.min.height, self.max.height),
        )
    }

    /// Constraints with `max.width = w`. `min.width` is also clamped to `w`
    /// to preserve `min <= max`.
    #[must_use]
    pub fn with_width(self, w: u32) -> Self {
        Self {
            min: Size::new(self.min.width.min(w), self.min.height),
            max: Size::new(w, self.max.height),
        }
    }

    /// Constraints with `max.height = h`. `min.height` is also clamped.
    #[must_use]
    pub fn with_height(self, h: u32) -> Self {
        Self {
            min: Size::new(self.min.width, self.min.height.min(h)),
            max: Size::new(self.max.width, h),
        }
    }

    /// Subtract `dx, dy` from both min and max (saturating). Used to apply
    /// padding before measuring a child.
    #[must_use]
    pub fn shrink(self, dx: u32, dy: u32) -> Self {
        Self {
            min: Size::new(
                self.min.width.saturating_sub(dx),
                self.min.height.saturating_sub(dy),
            ),
            max: Size::new(
                self.max.width.saturating_sub(dx),
                self.max.height.saturating_sub(dy),
            ),
        }
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self::unbounded()
    }
}

/// Per-axis sizing intent. Pattern modeled on iced / libcosmic.
///
/// Containers use this to decide how much room each child gets before
/// they call `measure` and `arrange`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Length {
    /// Take exactly this many pixels.
    Fixed(u32),
    /// Ask the child what it wants (calls `measure` with loose
    /// constraints) and give it that.
    Shrink,
    /// Take all remaining space. Equivalent to `FillPortion(1)`.
    Fill,
    /// Take a share of remaining space proportional to `portion`.
    /// Multiple `FillPortion` (and `Fill`) siblings split the residual
    /// in proportion to their portion sums.
    FillPortion(u32),
}

impl Length {
    /// Returns the explicit pixel count if this is `Fixed`, else `None`.
    /// Convenience for the common "is this a fixed slot" check during
    /// container layout.
    #[must_use]
    pub fn fixed(self) -> Option<u32> {
        match self {
            Length::Fixed(n) => Some(n),
            _ => None,
        }
    }

    /// Portion weight for `Fill` / `FillPortion`. `0` for non-flex
    /// variants.
    #[must_use]
    pub fn portion(self) -> u32 {
        match self {
            Length::Fill => 1,
            Length::FillPortion(p) => p.max(1),
            _ => 0,
        }
    }

    /// Resolve to a concrete pixel count given the widget's intrinsic
    /// size on this axis and the max available from constraints.
    #[must_use]
    pub fn resolve(self, intrinsic: u32, max: u32) -> u32 {
        match self {
            Length::Fixed(n) => n.min(max),
            Length::Shrink => intrinsic.min(max),
            Length::Fill | Length::FillPortion(_) => max,
        }
    }
}

impl From<u32> for Length {
    fn from(n: u32) -> Self {
        Length::Fixed(n)
    }
}

impl From<u16> for Length {
    fn from(n: u16) -> Self {
        Length::Fixed(n as u32)
    }
}

// Accept default integer literals (`i32`) and `usize` so call sites
// can write `.width(64)` without a type suffix. Negative values are
// clamped to zero — negative widths/heights are nonsensical.
impl From<i32> for Length {
    fn from(n: i32) -> Self {
        Length::Fixed(n.max(0) as u32)
    }
}

impl From<usize> for Length {
    fn from(n: usize) -> Self {
        Length::Fixed(n.min(u32::MAX as usize) as u32)
    }
}

/// Horizontal alignment for child or text content within a slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Horizontal {
    /// Align to the left edge.
    Left,
    /// Center horizontally.
    Center,
    /// Align to the right edge.
    Right,
}

/// Vertical alignment for child or text content within a slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Vertical {
    /// Align to the top edge.
    Top,
    /// Center vertically.
    Center,
    /// Align to the bottom edge.
    Bottom,
}
