use crate::{
    TextBlobId,
    geometry::{Point, Px, Rect},
    ids::FontId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::scene::Color;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font: FontId,
    pub size: Px,
    pub weight: FontWeight,
    pub slant: TextSlant,
    /// Optional line height override, in logical px.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<Px>,
    /// Optional tracking (letter spacing) override, in EM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing_em: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecorationLineStyle {
    #[default]
    Solid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnderlineStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default)]
    pub style: DecorationLineStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrikethroughStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default)]
    pub style: DecorationLineStyle,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TextShapingStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<FontId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<FontWeight>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slant: Option<TextSlant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing_em: Option<f32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TextPaintStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underline: Option<UnderlineStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<StrikethroughStyle>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TextSpan {
    /// Span length in UTF-8 bytes.
    pub len: usize,
    #[serde(default)]
    pub shaping: TextShapingStyle,
    #[serde(default)]
    pub paint: TextPaintStyle,
}

impl TextSpan {
    pub fn new(len: usize) -> Self {
        Self {
            len,
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        }
    }
}

impl TextShapingStyle {
    pub fn with_font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn with_slant(mut self, slant: TextSlant) -> Self {
        self.slant = Some(slant);
        self
    }

    pub fn with_letter_spacing_em(mut self, letter_spacing_em: f32) -> Self {
        self.letter_spacing_em = Some(letter_spacing_em);
        self
    }
}

impl TextPaintStyle {
    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn with_underline(mut self, underline: UnderlineStyle) -> Self {
        self.underline = Some(underline);
        self
    }

    pub fn with_strikethrough(mut self, strikethrough: StrikethroughStyle) -> Self {
        self.strikethrough = Some(strikethrough);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributedText {
    pub text: Arc<str>,
    pub spans: Arc<[TextSpan]>,
}

fn spans_are_valid(text: &str, spans: &[TextSpan]) -> bool {
    let mut offset = 0usize;
    for span in spans {
        let end = offset.saturating_add(span.len);
        if end > text.len() {
            return false;
        }
        if !text.is_char_boundary(offset) || !text.is_char_boundary(end) {
            return false;
        }
        offset = end;
    }
    offset == text.len()
}

impl AttributedText {
    pub fn new(text: impl Into<Arc<str>>, spans: impl Into<Arc<[TextSpan]>>) -> Self {
        let text: Arc<str> = text.into();
        let spans: Arc<[TextSpan]> = spans.into();
        debug_assert!(spans_are_valid(text.as_ref(), spans.as_ref()));
        Self { text, spans }
    }

    pub fn is_valid(&self) -> bool {
        spans_are_valid(self.text.as_ref(), self.spans.as_ref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextInputRef<'a> {
    Plain {
        text: &'a str,
        style: &'a TextStyle,
    },
    Attributed {
        text: &'a str,
        base: &'a TextStyle,
        spans: &'a [TextSpan],
    },
}

impl<'a> TextInputRef<'a> {
    pub fn plain(text: &'a str, style: &'a TextStyle) -> Self {
        Self::Plain { text, style }
    }

    pub fn attributed(text: &'a str, base: &'a TextStyle, spans: &'a [TextSpan]) -> Self {
        debug_assert!(spans_are_valid(text, spans));
        Self::Attributed { text, base, spans }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TextInput {
    Plain {
        text: Arc<str>,
        style: TextStyle,
    },
    Attributed {
        text: Arc<str>,
        base: TextStyle,
        spans: Arc<[TextSpan]>,
    },
}

impl TextInput {
    pub fn plain(text: impl Into<Arc<str>>, style: TextStyle) -> Self {
        Self::Plain {
            text: text.into(),
            style,
        }
    }

    pub fn attributed(
        text: impl Into<Arc<str>>,
        base: TextStyle,
        spans: impl Into<Arc<[TextSpan]>>,
    ) -> Self {
        Self::Attributed {
            text: text.into(),
            base,
            spans: spans.into(),
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Plain { text, .. } => text.as_ref(),
            Self::Attributed { text, .. } => text.as_ref(),
        }
    }
}

pub trait TextService {
    fn prepare(
        &mut self,
        input: &TextInput,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics);

    fn prepare_str(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let input = TextInput::plain(Arc::<str>::from(text), style.clone());
        self.prepare(&input, constraints)
    }

    fn prepare_rich(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let input =
            TextInput::attributed(rich.text.clone(), base_style.clone(), rich.spans.clone());
        self.prepare(&input, constraints)
    }

    fn measure(&mut self, input: &TextInput, constraints: TextConstraints) -> TextMetrics {
        let (blob, metrics) = self.prepare(input, constraints);
        self.release(blob);
        metrics
    }

    fn measure_str(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let input = TextInput::plain(Arc::<str>::from(text), style.clone());
        self.measure(&input, constraints)
    }

    fn measure_rich(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let (blob, metrics) = self.prepare_rich(rich, base_style, constraints);
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
