//! [`CornerRadii`]: design-system corner-radius scale.

/// Design-system corner-radius scale, in pixels.
///
/// Widgets reference these values rather than hard-coding radii. On
/// embedded displays with limited fill primitives, "small" radii (1–2 px)
/// are typically faked by chamfering one or two corner pixels rather than
/// drawing arcs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CornerRadii {
    /// 0 px — square corners.
    pub radius_0: u16,
    /// Extra-small (1 px).
    pub radius_xs: u16,
    /// Small (2 px).
    pub radius_s: u16,
    /// Medium (4 px).
    pub radius_m: u16,
    /// Large (6 px).
    pub radius_l: u16,
    /// Extra-large (8 px).
    pub radius_xl: u16,
}

impl CornerRadii {
    /// Default scale.
    pub const fn default_small() -> Self {
        Self {
            radius_0: 0,
            radius_xs: 1,
            radius_s: 2,
            radius_m: 4,
            radius_l: 6,
            radius_xl: 8,
        }
    }
}

impl Default for CornerRadii {
    fn default() -> Self {
        Self::default_small()
    }
}
