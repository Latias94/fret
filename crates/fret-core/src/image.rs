/// Color space metadata for image resources.
///
/// This is a portable, contract-level value type used across the runtime/renderer boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageColorSpace {
    Srgb,
    Linear,
}

impl ImageColorSpace {
    pub const fn to_color_info(self) -> ImageColorInfo {
        match self {
            Self::Srgb => ImageColorInfo::srgb_rgba(),
            Self::Linear => ImageColorInfo::linear_rgba(),
        }
    }
}

/// Stable metadata describing how to interpret pixel bytes for streaming images (ADR 0124).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageColorInfo {
    pub encoding: ImageEncoding,
    pub range: ColorRange,
    pub matrix: YuvMatrix,
    pub primaries: ColorPrimaries,
    pub transfer: TransferFunction,
    pub chroma_siting: Option<ChromaSiting>,
}

impl ImageColorInfo {
    pub const fn srgb_rgba() -> Self {
        Self {
            encoding: ImageEncoding::Srgb,
            range: ColorRange::Full,
            matrix: YuvMatrix::Bt709,
            primaries: ColorPrimaries::Bt709,
            transfer: TransferFunction::Srgb,
            chroma_siting: None,
        }
    }

    pub const fn linear_rgba() -> Self {
        Self {
            encoding: ImageEncoding::Linear,
            range: ColorRange::Full,
            matrix: YuvMatrix::Bt709,
            primaries: ColorPrimaries::Bt709,
            transfer: TransferFunction::Linear,
            chroma_siting: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageEncoding {
    Srgb,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorRange {
    Full,
    Limited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YuvMatrix {
    Bt601,
    Bt709,
    Bt2020,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorPrimaries {
    Bt709,
    Bt2020,
    DisplayP3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferFunction {
    Linear,
    Srgb,
    Bt709,
    Pq,
    Hlg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromaSiting {
    Center,
    Left,
    TopLeft,
}

/// Explicit alpha semantics for image updates (ADR 0124 / ADR 0040).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlphaMode {
    Opaque,
    Premultiplied,
    Straight,
}
