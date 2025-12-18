use crate::{TextBlobId, geometry::Px, ids::FontId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextWrap {
    None,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextConstraints {
    pub max_width: Option<Px>,
    pub wrap: TextWrap,
    /// Window/device scale factor used for rasterization and caching.
    ///
    /// UI/layout coordinates remain in logical pixels. Implementations should rasterize at
    /// `style.size * scale_factor` (and any other scale-dependent parameters), then return metrics
    /// back in logical units.
    pub scale_factor: f32,
}

impl Default for TextConstraints {
    fn default() -> Self {
        Self {
            max_width: None,
            wrap: TextWrap::Word,
            scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub font: FontId,
    pub size: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    pub size: crate::Size,
    pub baseline: Px,
}

pub trait TextService {
    fn prepare(
        &mut self,
        text: &str,
        style: TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics);

    fn release(&mut self, blob: TextBlobId);
}
