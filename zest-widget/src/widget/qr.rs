//! Passive QR code widget. Encodes owned UTF-8 text or arbitrary bytes and
//! renders a centered, integer-scaled symbol with a configurable quiet zone.
//!
//! The host rebuilds the widget each frame, so encoding happens when the
//! widget is constructed or when [`Qr::ecc`] changes. Encoding failures never
//! panic; query them via [`Qr::error`].

use super::Widget;
use alloc::{string::String, vec, vec::Vec};
use core::marker::PhantomData;
use embedded_graphics::{pixelcolor::PixelColor, prelude::*, primitives::Rectangle};
use qrcodegen_no_heap::{DataTooLong, QrCode, QrCodeEcc, Version};
use zest_core::{Constraints, Length, RenderError, Renderer, TouchPhase};
use zest_theme::Theme;

/// Default quiet-zone width in modules, per the QR specification.
const DEFAULT_QUIET_ZONE: u32 = 4;

/// QR error correction level.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum EccLevel {
    /// Roughly 7% error correction.
    Low,
    /// Roughly 15% error correction.
    #[default]
    Medium,
    /// Roughly 25% error correction.
    Quartile,
    /// Roughly 30% error correction.
    High,
}

impl EccLevel {
    const fn into_qr(self) -> QrCodeEcc {
        match self {
            Self::Low => QrCodeEcc::Low,
            Self::Medium => QrCodeEcc::Medium,
            Self::Quartile => QrCodeEcc::Quartile,
            Self::High => QrCodeEcc::High,
        }
    }
}

/// QR encode failure surfaced by [`Qr::error`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QrError {
    /// The payload is too large for a byte-mode segment.
    SegmentTooLong,
    /// The payload exceeds the QR capacity for the selected error correction.
    DataOverCapacity {
        /// Bits needed to encode the payload.
        used_bits: usize,
        /// Bits available in the chosen symbol version.
        capacity_bits: usize,
    },
}

impl From<DataTooLong> for QrError {
    fn from(err: DataTooLong) -> Self {
        match err {
            DataTooLong::SegmentTooLong => Self::SegmentTooLong,
            DataTooLong::DataOverCapacity(used_bits, capacity_bits) => Self::DataOverCapacity {
                used_bits,
                capacity_bits,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EncodedQr {
    size: u32,
    modules: Vec<u8>,
}

impl EncodedQr {
    fn encode(data: &[u8], ecc: EccLevel) -> Result<Self, QrError> {
        let mut temp = vec![0u8; Version::MAX.buffer_len()];
        let mut out = vec![0u8; Version::MAX.buffer_len()];

        if data.len() > temp.len() {
            return Err(QrError::SegmentTooLong);
        }
        temp[..data.len()].copy_from_slice(data);

        let qr = QrCode::encode_binary(
            &mut temp,
            data.len(),
            &mut out,
            ecc.into_qr(),
            Version::MIN,
            Version::MAX,
            None,
            true,
        )
        .map_err(QrError::from)?;

        let size = qr.size() as u32;
        let mut modules = vec![0u8; ((size * size) as usize).div_ceil(8)];
        for y in 0..size {
            for x in 0..size {
                if qr.get_module(x as i32, y as i32) {
                    let index = (y * size + x) as usize;
                    modules[index >> 3] |= 1u8 << (index & 7);
                }
            }
        }

        Ok(Self { size, modules })
    }

    fn side_modules(&self, quiet_zone: u32) -> u32 {
        self.size.saturating_add(quiet_zone.saturating_mul(2))
    }

    fn is_dark(&self, x: u32, y: u32) -> bool {
        let index = (y * self.size + x) as usize;
        (self.modules[index >> 3] >> (index & 7)) & 1 != 0
    }
}

/// Passive QR code widget with builder-style sizing, ECC, quiet zone, and colors.
pub struct Qr<C: PixelColor, M: Clone> {
    rect: Rectangle,
    data: Vec<u8>,
    encoded: Result<EncodedQr, QrError>,
    ecc: EccLevel,
    quiet_zone: u32,
    dark: Option<C>,
    light: Option<C>,
    width: Length,
    height: Length,
    _phantom: PhantomData<M>,
}

impl<C: PixelColor, M: Clone> Qr<C, M> {
    /// New QR code from UTF-8 text. The payload is encoded in byte mode.
    pub fn new(data: impl Into<String>) -> Self {
        Self::from_bytes(data.into().into_bytes())
    }

    /// New QR code from arbitrary bytes. The payload is encoded in byte mode.
    pub fn from_bytes(data: impl Into<Vec<u8>>) -> Self {
        let data = data.into();
        let ecc = EccLevel::default();
        let encoded = EncodedQr::encode(&data, ecc);
        Self {
            rect: Rectangle::zero(),
            data,
            encoded,
            ecc,
            quiet_zone: DEFAULT_QUIET_ZONE,
            dark: None,
            light: None,
            width: Length::Shrink,
            height: Length::Shrink,
            _phantom: PhantomData,
        }
    }

    /// Width sizing intent.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Height sizing intent.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Error correction level (default: `Medium`).
    #[must_use]
    pub fn ecc(mut self, ecc: EccLevel) -> Self {
        self.ecc = ecc;
        self.reencode();
        self
    }

    /// Quiet zone width in modules (default: 4).
    #[must_use]
    pub fn quiet_zone(mut self, quiet_zone: u32) -> Self {
        self.quiet_zone = quiet_zone;
        self
    }

    /// Override the dark-module color (default: `theme.background.on_base`).
    #[must_use]
    pub fn dark(mut self, color: C) -> Self {
        self.dark = Some(color);
        self
    }

    /// Override the light-module color (default: `theme.background.base`).
    #[must_use]
    pub fn light(mut self, color: C) -> Self {
        self.light = Some(color);
        self
    }

    /// Returns the current encode error, if any.
    pub fn error(&self) -> Option<&QrError> {
        self.encoded.as_ref().err()
    }

    fn reencode(&mut self) {
        self.encoded = EncodedQr::encode(&self.data, self.ecc);
    }

    fn intrinsic_side(&self) -> u32 {
        match self.encoded.as_ref() {
            Ok(encoded) => encoded.side_modules(self.quiet_zone),
            Err(_) => 0,
        }
    }
}

impl<C: PixelColor, M: Clone> Widget<C, M> for Qr<C, M> {
    fn measure(&mut self, constraints: Constraints) -> Size {
        let intrinsic = self.intrinsic_side();
        let width = self.width.resolve(intrinsic, constraints.max.width);
        let height = self.height.resolve(intrinsic, constraints.max.height);
        constraints.clamp(Size::new(width, height))
    }

    fn preferred_size(&self) -> (Length, Length) {
        (self.width, self.height)
    }

    fn arrange(&mut self, rect: Rectangle) {
        self.rect = rect;
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }

    fn handle_touch(&mut self, _point: Point, _phase: TouchPhase) -> Option<M> {
        None
    }

    fn draw<'t>(
        &self,
        renderer: &mut dyn Renderer<C>,
        theme: &Theme<'t, C>,
    ) -> Result<(), RenderError> {
        let Ok(encoded) = self.encoded.as_ref() else {
            return Ok(());
        };

        let side_modules = encoded.side_modules(self.quiet_zone);
        if side_modules == 0 {
            return Ok(());
        }

        let module_px =
            (self.rect.size.width / side_modules).min(self.rect.size.height / side_modules);
        if module_px == 0 {
            return Ok(());
        }

        let qr_px = side_modules * module_px;
        let dx = ((self.rect.size.width - qr_px) / 2) as i32;
        let dy = ((self.rect.size.height - qr_px) / 2) as i32;
        let origin = self.rect.top_left + Point::new(dx, dy);
        let light = self.light.unwrap_or(theme.background.base);
        let dark = self.dark.unwrap_or(theme.background.on_base);

        renderer.fill_rect(Rectangle::new(origin, Size::new(qr_px, qr_px)), light)?;

        for y in 0..encoded.size {
            let y_px = origin.y + ((y + self.quiet_zone) * module_px) as i32;
            let mut x = 0;
            while x < encoded.size {
                if !encoded.is_dark(x, y) {
                    x += 1;
                    continue;
                }

                let run_start = x;
                x += 1;
                while x < encoded.size && encoded.is_dark(x, y) {
                    x += 1;
                }

                let x_px = origin.x + ((run_start + self.quiet_zone) * module_px) as i32;
                let run_w = (x - run_start) * module_px;
                renderer.fill_rect(
                    Rectangle::new(Point::new(x_px, y_px), Size::new(run_w, module_px)),
                    dark,
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use embedded_graphics::{mono_font::MonoFont, pixelcolor::BinaryColor, text::Alignment};
    use zest_core::Renderer;
    use zest_theme::{Theme, convert_theme, theme::dark};

    struct RecordingRenderer {
        fills: Vec<Rectangle>,
    }

    impl RecordingRenderer {
        fn new() -> Self {
            Self { fills: Vec::new() }
        }
    }

    impl Renderer<BinaryColor> for RecordingRenderer {
        fn fill_rect(&mut self, rect: Rectangle, _color: BinaryColor) -> Result<(), RenderError> {
            self.fills.push(rect);
            Ok(())
        }

        fn stroke_rect(
            &mut self,
            _rect: Rectangle,
            _color: BinaryColor,
        ) -> Result<(), RenderError> {
            Ok(())
        }

        fn fill_circle(
            &mut self,
            _center: Point,
            _radius: u32,
            _color: BinaryColor,
        ) -> Result<(), RenderError> {
            Ok(())
        }

        fn stroke_line(
            &mut self,
            _start: Point,
            _end: Point,
            _color: BinaryColor,
            _width: u32,
        ) -> Result<(), RenderError> {
            Ok(())
        }

        fn draw_text(
            &mut self,
            _text: &str,
            _position: Point,
            _font: &MonoFont<'_>,
            _color: BinaryColor,
            _alignment: Alignment,
        ) -> Result<(), RenderError> {
            Ok(())
        }
    }

    fn theme() -> Theme<'static, BinaryColor> {
        convert_theme(&dark::THEME)
    }

    #[test]
    fn oversize_payload_surfaces_error() {
        let qr = Qr::<BinaryColor, ()>::from_bytes(vec![0x41; 4096]);
        assert!(matches!(
            qr.error(),
            Some(QrError::SegmentTooLong | QrError::DataOverCapacity { .. })
        ));
    }

    #[test]
    fn draw_centers_a_square_integer_scaled_symbol() {
        let mut qr = Qr::<BinaryColor, ()>::new("https://bhh32.com")
            .quiet_zone(4)
            .width(Length::Fixed(120))
            .height(Length::Fixed(100));
        qr.arrange(Rectangle::new(Point::new(10, 20), Size::new(120, 100)));

        let mut renderer = RecordingRenderer::new();
        qr.draw(&mut renderer, &theme()).unwrap();

        let bg = renderer.fills.first().copied().unwrap();
        assert_eq!(bg.size.width, bg.size.height);
        assert_eq!(bg.top_left.y, 20);
        assert!(bg.top_left.x > 10);
        assert!(renderer.fills.len() > 1);
    }
}
