use crate::{
    TextBlobId,
    geometry::{Point, Px, Rect},
    ids::FontId,
};
use serde::{Deserialize, Serialize};

/// Overrides for the default font family selection used by the text system.
///
/// This is intended to be persisted in settings/config files and applied by the host/runner.
/// It configures the three generic families used by `TextStyle.font` (`Ui`/`Serif`/`Monospace`).
///
/// Notes:
/// - Entries are treated as ordered "try this first" candidates; backends will pick the first
///   installed family name and ignore unknown ones.
/// - This does not attempt to model per-script fallback chains yet (ADR 0029).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFontFamilyConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_sans: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_serif: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_mono: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const THIN: Self = Self(100);
    pub const EXTRA_LIGHT: Self = Self(200);
    pub const LIGHT: Self = Self(300);
    pub const NORMAL: Self = Self(400);
    pub const MEDIUM: Self = Self(500);
    pub const SEMIBOLD: Self = Self(600);
    pub const BOLD: Self = Self(700);
    pub const EXTRA_BOLD: Self = Self(800);
    pub const BLACK: Self = Self(900);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextWrap {
    None,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextConstraints {
    pub max_width: Option<Px>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
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
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font: FontId,
    pub size: Px,
    pub weight: FontWeight,
    pub slant: TextSlant,
    /// Optional line height override, in logical px.
    pub line_height: Option<Px>,
    /// Optional tracking (letter spacing) override, in EM.
    pub letter_spacing_em: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextSlant {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: FontId::default(),
            size: Px(13.0),
            weight: FontWeight::NORMAL,
            slant: TextSlant::Normal,
            line_height: None,
            letter_spacing_em: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    pub size: crate::Size,
    pub baseline: Px,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaretAffinity {
    Upstream,
    Downstream,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTestResult {
    pub index: usize,
    pub affinity: CaretAffinity,
}

pub trait TextService {
    fn prepare(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics);

    fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let (blob, metrics) = self.prepare(text, style, constraints);
        self.release(blob);
        metrics
    }

    /// Returns the X offset (in logical px) of the caret at `index` within the prepared text blob.
    ///
    /// Coordinate space: relative to the text origin (x=0 at the beginning of the line).
    ///
    /// Notes:
    /// - `index` is a byte offset into the UTF-8 text, clamped to valid char boundaries (ADR 0044).
    /// - Implementations may clamp to the nearest representable caret position.
    fn caret_x(&mut self, _blob: TextBlobId, _index: usize) -> Px {
        Px(0.0)
    }

    /// Performs hit-testing for a single-line text blob and returns the nearest caret byte index.
    ///
    /// Coordinate space: `x` is relative to the text origin (x=0 at the beginning of the line).
    fn hit_test_x(&mut self, _blob: TextBlobId, _x: Px) -> usize {
        0
    }

    /// Computes selection rectangles for a single-line selection range.
    ///
    /// Coordinate space: rects are relative to the text origin (x=0, y=0 at top of text box).
    fn selection_rects(&mut self, _blob: TextBlobId, _range: (usize, usize), _out: &mut Vec<Rect>) {
    }

    /// Extracts the precomputed caret stop table (byte index -> x offset) for a single-line blob.
    ///
    /// This is primarily intended for UI hit-testing in event handlers, which do not have access
    /// to the text service.
    fn caret_stops(&mut self, _blob: TextBlobId, _out: &mut Vec<(usize, Px)>) {}

    /// Returns the caret rectangle (in logical px) for the given `index`.
    ///
    /// Coordinate space: rect is relative to the text origin (x=0, y=0 at the top of the text box).
    ///
    /// Notes:
    /// - Single-line implementations may ignore affinity.
    /// - Multi-line implementations should use affinity to disambiguate positions at line breaks.
    fn caret_rect(&mut self, _blob: TextBlobId, _index: usize, _affinity: CaretAffinity) -> Rect {
        Rect::default()
    }

    /// Hit-test a point in the text's local coordinate space and return a caret index and affinity.
    ///
    /// Coordinate space: `point` is relative to the text origin (x=0, y=0 at the top of the text box).
    fn hit_test_point(&mut self, _blob: TextBlobId, _point: Point) -> HitTestResult {
        HitTestResult {
            index: 0,
            affinity: CaretAffinity::Downstream,
        }
    }

    fn release(&mut self, blob: TextBlobId);
}
