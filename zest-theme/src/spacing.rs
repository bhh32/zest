//! [`Spacing`]: design-system spacing scale in pixels.

/// Design-system spacing scale, in pixels.
///
/// Mirrors libcosmic's `space_xxxs` through `space_xxxl` scale. Widgets
/// reference these values rather than hard-coding pixel counts, so the
/// entire UI scales coherently when a theme is swapped.
///
/// Default values are tuned for a 320×240 display at 1× density. For
/// larger displays consider scaling proportionally.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spacing {
    /// 0 px — for explicit "no gap" placement.
    pub space_none: u16,
    /// Extra-extra-extra-small (1 px).
    pub space_xxxs: u16,
    /// Extra-extra-small (2 px).
    pub space_xxs: u16,
    /// Extra-small (4 px).
    pub space_xs: u16,
    /// Small (8 px).
    pub space_s: u16,
    /// Medium (12 px).
    pub space_m: u16,
    /// Large (16 px).
    pub space_l: u16,
    /// Extra-large (24 px).
    pub space_xl: u16,
    /// Extra-extra-large (32 px).
    pub space_xxl: u16,
    /// Extra-extra-extra-large (48 px).
    pub space_xxxl: u16,
}

impl Spacing {
    /// Default scale tuned for 320×240 displays.
    pub const fn default_small() -> Self {
        Self {
            space_none: 0,
            space_xxxs: 1,
            space_xxs: 2,
            space_xs: 4,
            space_s: 8,
            space_m: 12,
            space_l: 16,
            space_xl: 24,
            space_xxl: 32,
            space_xxxl: 48,
        }
    }
}

impl Default for Spacing {
    fn default() -> Self {
        Self::default_small()
    }
}
