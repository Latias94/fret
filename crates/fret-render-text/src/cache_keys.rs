use fret_core::{TextConstraints, TextOverflow, TextSlant, TextStyle, TextWrap};
use std::hash::{Hash as _, Hasher as _};

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextMeasureKey {
    pub font: fret_core::FontId,
    pub font_stack_key: u64,
    pub size_bits: u32,
    pub weight: u16,
    pub slant: u8,
    pub line_height_bits: Option<u32>,
    pub letter_spacing_bits: Option<u32>,
    pub max_width_bits: Option<u32>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub scale_bits: u32,
}

impl TextMeasureKey {
    pub fn new(style: &TextStyle, constraints: TextConstraints, font_stack_key: u64) -> Self {
        let max_width_bits = match constraints.wrap {
            // `TextWrap::None` does not change shaping results based on width; callers clamp or
            // apply overflow policy at higher levels. Normalize away width so repeated measurements
            // (e.g. layout engine intrinsic probes) can reuse cached metrics.
            TextWrap::None => None,
            TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme => {
                constraints.max_width.map(|w| w.0.to_bits())
            }
        };
        Self {
            font: style.font.clone(),
            font_stack_key,
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
            slant: match style.slant {
                TextSlant::Normal => 0,
                TextSlant::Italic => 1,
                TextSlant::Oblique => 2,
            },
            line_height_bits: style.line_height.map(|px| px.0.to_bits()),
            letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_bits: constraints.scale_factor.to_bits(),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextMeasureShapingKey {
    pub text_hash: u64,
    pub text_len: usize,
    pub spans_shaping_key: u64,
    pub font: fret_core::FontId,
    pub font_stack_key: u64,
    pub size_bits: u32,
    pub weight: u16,
    pub slant: u8,
    pub line_height_bits: Option<u32>,
    pub letter_spacing_bits: Option<u32>,
    pub scale_bits: u32,
}

#[doc(hidden)]
pub fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
