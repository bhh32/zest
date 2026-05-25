//! The [`ScreenView`] trait: a screen's name, theme, and view tree.

use crate::widget::Element;
use embedded_graphics::pixelcolor::PixelColor;
use zest_theme::Theme;

/// Per-screen contract.
pub trait ScreenView<C: PixelColor, M: Clone> {
    /// Human-readable screen name.
    fn name(&self) -> &'static str;

    /// Build a fresh widget tree from `&self`. Called once per frame;
    /// the returned tree is owned for that frame and dropped at its end.
    fn view(&self) -> Element<'_, C, M>;

    /// Theme used to draw this screen.
    fn theme(&self) -> &Theme<'_, C>;
}
