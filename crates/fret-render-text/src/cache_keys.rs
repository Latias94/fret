use fret_core::{
    AttributedText, TextAlign, TextConstraints, TextOverflow, TextSlant, TextSpan, TextStyle,
    TextWrap,
};
use std::{
    hash::{Hash as _, Hasher as _},
    sync::Arc,
};

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextBlobKey {
    pub text: Arc<str>,
    pub spans_shaping_key: u64,
    pub spans_paint_key: u64,
    pub backend: u8,
    pub font: fret_core::FontId,
    pub font_stack_key: u64,
    pub size_bits: u32,
    pub weight: u16,
    pub slant: u8,
    pub line_height_bits: Option<u32>,
    pub line_height_em_bits: Option<u32>,
    pub line_height_policy: u8,
    pub leading_distribution: u8,
    pub strut_force: u8,
    pub strut_font: Option<fret_core::FontId>,
    pub strut_size_bits: Option<u32>,
    pub strut_line_height_bits: Option<u32>,
    pub strut_line_height_em_bits: Option<u32>,
    pub strut_leading_distribution: Option<u8>,
    pub letter_spacing_bits: Option<u32>,
    pub max_width_bits: Option<u32>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub align: u8,
    pub scale_bits: u32,
}

impl TextBlobKey {
    pub fn new(
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        font_stack_key: u64,
    ) -> Self {
        let max_width_bits = match constraints.wrap {
            // `TextWrap::None` does not change shaping results based on width unless we need to
            // materialize an overflow policy (ellipsis) or align within a wider box.
            TextWrap::None if constraints.overflow != TextOverflow::Ellipsis => match constraints
                .align
            {
                TextAlign::Start => None,
                TextAlign::Center | TextAlign::End => constraints.max_width.map(|w| w.0.to_bits()),
            },
            _ => constraints.max_width.map(|w| w.0.to_bits()),
        };
        let leading_distribution = match style.leading_distribution {
            fret_core::text::TextLeadingDistribution::Even => 0,
            fret_core::text::TextLeadingDistribution::Proportional => 1,
        };
        let (
            strut_force,
            strut_font,
            strut_size_bits,
            strut_line_height_bits,
            strut_line_height_em_bits,
            strut_leading_distribution,
        ) = if let Some(strut) = style.strut_style.as_ref() {
            (
                if strut.force { 1 } else { 0 },
                strut.font.clone(),
                strut.size.map(|px| px.0.to_bits()),
                strut.line_height.map(|px| px.0.to_bits()),
                strut.line_height_em.map(|v| v.to_bits()),
                strut.leading_distribution.map(|d| match d {
                    fret_core::text::TextLeadingDistribution::Even => 0,
                    fret_core::text::TextLeadingDistribution::Proportional => 1,
                }),
            )
        } else {
            (0, None, None, None, None, None)
        };

        Self {
            text: Arc::<str>::from(text),
            spans_shaping_key: 0,
            spans_paint_key: 0,
            backend: 0,
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
            line_height_em_bits: style.line_height_em.map(|v| v.to_bits()),
            line_height_policy: match style.line_height_policy {
                fret_core::TextLineHeightPolicy::ExpandToFit => 0,
                fret_core::TextLineHeightPolicy::FixedFromStyle => 1,
            },
            leading_distribution,
            strut_force,
            strut_font,
            strut_size_bits,
            strut_line_height_bits,
            strut_line_height_em_bits,
            strut_leading_distribution,
            letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            align: match constraints.align {
                TextAlign::Start => 0,
                TextAlign::Center => 1,
                TextAlign::End => 2,
            },
            scale_bits: constraints.scale_factor.to_bits(),
        }
    }

    pub fn new_attributed(
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
        font_stack_key: u64,
    ) -> Self {
        let mut out = Self::new(rich.text.as_ref(), base_style, constraints, font_stack_key);
        out.spans_shaping_key = spans_shaping_fingerprint(&rich.spans);
        out.spans_paint_key = spans_paint_fingerprint(&rich.spans);
        out
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextShapeKey {
    pub text: Arc<str>,
    pub spans_shaping_key: u64,
    pub backend: u8,
    pub font: fret_core::FontId,
    pub font_stack_key: u64,
    pub size_bits: u32,
    pub weight: u16,
    pub slant: u8,
    pub line_height_bits: Option<u32>,
    pub line_height_em_bits: Option<u32>,
    pub line_height_policy: u8,
    pub leading_distribution: u8,
    pub strut_force: u8,
    pub strut_font: Option<fret_core::FontId>,
    pub strut_size_bits: Option<u32>,
    pub strut_line_height_bits: Option<u32>,
    pub strut_line_height_em_bits: Option<u32>,
    pub strut_leading_distribution: Option<u8>,
    pub letter_spacing_bits: Option<u32>,
    pub max_width_bits: Option<u32>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub align: u8,
    pub scale_bits: u32,
}

impl TextShapeKey {
    pub fn from_blob_key(key: &TextBlobKey) -> Self {
        Self {
            text: key.text.clone(),
            spans_shaping_key: key.spans_shaping_key,
            backend: key.backend,
            font: key.font.clone(),
            font_stack_key: key.font_stack_key,
            size_bits: key.size_bits,
            weight: key.weight,
            slant: key.slant,
            line_height_bits: key.line_height_bits,
            line_height_em_bits: key.line_height_em_bits,
            line_height_policy: key.line_height_policy,
            leading_distribution: key.leading_distribution,
            strut_force: key.strut_force,
            strut_font: key.strut_font.clone(),
            strut_size_bits: key.strut_size_bits,
            strut_line_height_bits: key.strut_line_height_bits,
            strut_line_height_em_bits: key.strut_line_height_em_bits,
            strut_leading_distribution: key.strut_leading_distribution,
            letter_spacing_bits: key.letter_spacing_bits,
            max_width_bits: key.max_width_bits,
            wrap: key.wrap,
            overflow: key.overflow,
            align: key.align,
            scale_bits: key.scale_bits,
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextMeasureKey {
    pub font: fret_core::FontId,
    pub font_stack_key: u64,
    pub size_bits: u32,
    pub weight: u16,
    pub slant: u8,
    pub line_height_bits: Option<u32>,
    pub line_height_em_bits: Option<u32>,
    pub line_height_policy: u8,
    pub leading_distribution: u8,
    pub strut_force: u8,
    pub strut_font: Option<fret_core::FontId>,
    pub strut_size_bits: Option<u32>,
    pub strut_line_height_bits: Option<u32>,
    pub strut_line_height_em_bits: Option<u32>,
    pub strut_leading_distribution: Option<u8>,
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
            TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme => {
                constraints.max_width.map(|w| w.0.to_bits())
            }
        };
        let leading_distribution = match style.leading_distribution {
            fret_core::text::TextLeadingDistribution::Even => 0,
            fret_core::text::TextLeadingDistribution::Proportional => 1,
        };
        let (
            strut_force,
            strut_font,
            strut_size_bits,
            strut_line_height_bits,
            strut_line_height_em_bits,
            strut_leading_distribution,
        ) = if let Some(strut) = style.strut_style.as_ref() {
            (
                if strut.force { 1 } else { 0 },
                strut.font.clone(),
                strut.size.map(|px| px.0.to_bits()),
                strut.line_height.map(|px| px.0.to_bits()),
                strut.line_height_em.map(|v| v.to_bits()),
                strut.leading_distribution.map(|d| match d {
                    fret_core::text::TextLeadingDistribution::Even => 0,
                    fret_core::text::TextLeadingDistribution::Proportional => 1,
                }),
            )
        } else {
            (0, None, None, None, None, None)
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
            line_height_em_bits: style.line_height_em.map(|v| v.to_bits()),
            line_height_policy: match style.line_height_policy {
                fret_core::TextLineHeightPolicy::ExpandToFit => 0,
                fret_core::TextLineHeightPolicy::FixedFromStyle => 1,
            },
            leading_distribution,
            strut_force,
            strut_font,
            strut_size_bits,
            strut_line_height_bits,
            strut_line_height_em_bits,
            strut_leading_distribution,
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
    pub line_height_em_bits: Option<u32>,
    pub line_height_policy: u8,
    pub leading_distribution: u8,
    pub strut_force: u8,
    pub strut_font: Option<fret_core::FontId>,
    pub strut_size_bits: Option<u32>,
    pub strut_line_height_bits: Option<u32>,
    pub strut_line_height_em_bits: Option<u32>,
    pub strut_leading_distribution: Option<u8>,
    pub letter_spacing_bits: Option<u32>,
    pub scale_bits: u32,
}

#[doc(hidden)]
pub fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[doc(hidden)]
pub fn spans_shaping_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.spans.shaping.v1".hash(&mut hasher);
    for s in spans {
        s.len.hash(&mut hasher);
        s.shaping.font.hash(&mut hasher);
        s.shaping.weight.hash(&mut hasher);
        s.shaping.slant.hash(&mut hasher);
        s.shaping
            .letter_spacing_em
            .map(|v| v.to_bits())
            .hash(&mut hasher);
        features_shaping_fingerprint(&mut hasher, &s.shaping.features);
        axes_shaping_fingerprint(&mut hasher, &s.shaping.axes);
    }
    hasher.finish()
}

fn features_shaping_fingerprint(
    hasher: &mut std::collections::hash_map::DefaultHasher,
    features: &[fret_core::TextFontFeatureSetting],
) {
    use std::collections::BTreeMap;

    if features.is_empty() {
        0u8.hash(hasher);
        return;
    }
    1u8.hash(hasher);

    let mut by_tag: BTreeMap<u32, u16> = BTreeMap::new();
    for feature in features {
        let tag = feature.tag.trim();
        if tag.is_empty() {
            continue;
        }
        let bytes = tag.as_bytes();
        if bytes.len() != 4 || !bytes.iter().all(u8::is_ascii) {
            continue;
        }

        let tag_u32 = (bytes[0] as u32) << 24
            | (bytes[1] as u32) << 16
            | (bytes[2] as u32) << 8
            | bytes[3] as u32;
        let value = feature.value.min(u32::from(u16::MAX)) as u16;
        by_tag.insert(tag_u32, value);
    }

    by_tag.len().hash(hasher);
    for (tag, value) in by_tag {
        tag.hash(hasher);
        value.hash(hasher);
    }
}

fn axes_shaping_fingerprint(
    hasher: &mut std::collections::hash_map::DefaultHasher,
    axes: &[fret_core::TextFontAxisSetting],
) {
    use std::collections::BTreeMap;

    if axes.is_empty() {
        0u8.hash(hasher);
        return;
    }
    1u8.hash(hasher);

    let mut by_tag: BTreeMap<u32, u32> = BTreeMap::new();
    for axis in axes {
        let tag = axis.tag.trim();
        if tag.is_empty() || !axis.value.is_finite() {
            continue;
        }
        let bytes = tag.as_bytes();
        if bytes.len() != 4 {
            continue;
        }

        let tag_u32 = (bytes[0] as u32) << 24
            | (bytes[1] as u32) << 16
            | (bytes[2] as u32) << 8
            | bytes[3] as u32;
        by_tag.insert(tag_u32, axis.value.to_bits());
    }

    by_tag.len().hash(hasher);
    for (tag, bits) in by_tag {
        tag.hash(hasher);
        bits.hash(hasher);
    }
}

#[doc(hidden)]
pub fn spans_paint_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.spans.paint.v0".hash(&mut hasher);
    for s in spans {
        s.len.hash(&mut hasher);
        paint_fingerprint_color(&mut hasher, s.paint.fg);
        paint_fingerprint_color(&mut hasher, s.paint.bg);

        match s.paint.underline.as_ref() {
            None => 0u8.hash(&mut hasher),
            Some(u) => {
                1u8.hash(&mut hasher);
                paint_fingerprint_color(&mut hasher, u.color);
                std::mem::discriminant(&u.style).hash(&mut hasher);
            }
        }

        match s.paint.strikethrough.as_ref() {
            None => 0u8.hash(&mut hasher),
            Some(st) => {
                1u8.hash(&mut hasher);
                paint_fingerprint_color(&mut hasher, st.color);
                std::mem::discriminant(&st.style).hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn paint_fingerprint_color(
    hasher: &mut std::collections::hash_map::DefaultHasher,
    color: Option<fret_core::Color>,
) {
    match color {
        None => 0u8.hash(hasher),
        Some(c) => {
            1u8.hash(hasher);
            c.r.to_bits().hash(hasher);
            c.g.to_bits().hash(hasher);
            c.b.to_bits().hash(hasher);
            c.a.to_bits().hash(hasher);
        }
    }
}
