use crate::ThemeSnapshot;
use fret_core::{
    AttributedText, FontId, Px, TextAlign, TextInput, TextOverflow, TextSlant, TextSpan, TextStyle,
    TextWrap,
};

pub(crate) fn default_text_style(theme: ThemeSnapshot) -> TextStyle {
    TextStyle {
        size: theme.metrics.font_size,
        line_height: Some(theme.metrics.font_line_height),
        ..Default::default()
    }
}

pub(crate) fn resolve_text_style(theme: ThemeSnapshot, explicit: Option<TextStyle>) -> TextStyle {
    explicit.unwrap_or_else(|| default_text_style(theme))
}

pub(crate) fn build_text_input_plain(text: std::sync::Arc<str>, style: TextStyle) -> TextInput {
    TextInput::plain(text, style)
}

pub(crate) fn build_text_input_attributed(
    rich: &fret_core::AttributedText,
    style: TextStyle,
) -> TextInput {
    TextInput::attributed(rich.text.clone(), style, rich.spans.clone())
}

pub(crate) fn text_wrap_none_measure_fingerprint_plain(
    text: &std::sync::Arc<str>,
    explicit_style: Option<&TextStyle>,
    theme_revision: u64,
    overflow: TextOverflow,
    align: TextAlign,
    scale_factor: f32,
    font_stack_key: u64,
) -> u64 {
    let mut state = 0u64;
    state = mix_u64(state, 1);
    state = mix_u64(state, text.as_ref().as_ptr() as usize as u64);
    state = mix_u64(state, text.len() as u64);
    state = mix_u64(state, font_stack_key);
    state = mix_u64(state, theme_revision);
    state = mix_f32(state, scale_factor);
    state = mix_text_wrap(state, TextWrap::None);
    state = mix_text_overflow(state, overflow);
    state = mix_text_align(state, align);
    if let Some(style) = explicit_style {
        state = mix_u64(state, 1);
        state = mix_text_style(state, style);
    } else {
        state = mix_u64(state, 0);
    }
    state
}

pub(crate) fn text_wrap_none_measure_fingerprint_rich(
    rich: &AttributedText,
    explicit_style: Option<&TextStyle>,
    theme_revision: u64,
    overflow: TextOverflow,
    align: TextAlign,
    scale_factor: f32,
    font_stack_key: u64,
) -> u64 {
    let mut state = 0u64;
    state = mix_u64(state, 2);
    state = mix_u64(state, rich.text.as_ref().as_ptr() as usize as u64);
    state = mix_u64(state, rich.text.len() as u64);
    state = mix_u64(state, spans_shaping_fingerprint(rich.spans.as_ref()));
    state = mix_u64(state, font_stack_key);
    state = mix_u64(state, theme_revision);
    state = mix_f32(state, scale_factor);
    state = mix_text_wrap(state, TextWrap::None);
    state = mix_text_overflow(state, overflow);
    state = mix_text_align(state, align);
    if let Some(style) = explicit_style {
        state = mix_u64(state, 1);
        state = mix_text_style(state, style);
    } else {
        state = mix_u64(state, 0);
    }
    state
}

fn mix_u64(mut state: u64, value: u64) -> u64 {
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

fn mix_px(state: u64, value: Px) -> u64 {
    mix_f32(state, value.0)
}

fn mix_option_px(mut state: u64, value: Option<Px>) -> u64 {
    state = mix_u64(state, value.is_some() as u64);
    if let Some(value) = value {
        state = mix_px(state, value);
    }
    state
}

fn mix_option_f32(mut state: u64, value: Option<f32>) -> u64 {
    state = mix_u64(state, value.is_some() as u64);
    if let Some(value) = value {
        state = mix_f32(state, value);
    }
    state
}

fn mix_bytes(mut state: u64, bytes: &[u8]) -> u64 {
    state = mix_u64(state, bytes.len() as u64);
    for &b in bytes {
        state = mix_u64(state, b as u64);
    }
    state
}

fn mix_font_id(mut state: u64, font: &FontId) -> u64 {
    match font {
        FontId::Ui => mix_u64(state, 1),
        FontId::Serif => mix_u64(state, 2),
        FontId::Monospace => mix_u64(state, 3),
        FontId::Family(name) => {
            state = mix_u64(state, 4);
            mix_bytes(state, name.as_bytes())
        }
    }
}

fn mix_text_style(mut state: u64, style: &TextStyle) -> u64 {
    state = mix_font_id(state, &style.font);
    state = mix_px(state, style.size);
    state = mix_u64(state, u64::from(style.weight.0));
    state = mix_text_slant(state, style.slant);
    state = mix_option_px(state, style.line_height);
    state = mix_option_f32(state, style.letter_spacing_em);
    state
}

fn mix_text_slant(state: u64, slant: TextSlant) -> u64 {
    mix_u64(
        state,
        match slant {
            TextSlant::Normal => 1,
            TextSlant::Italic => 2,
            TextSlant::Oblique => 3,
        },
    )
}

fn mix_text_wrap(state: u64, wrap: TextWrap) -> u64 {
    mix_u64(
        state,
        match wrap {
            TextWrap::None => 1,
            TextWrap::Word => 2,
            TextWrap::Grapheme => 3,
        },
    )
}

fn mix_text_overflow(state: u64, overflow: TextOverflow) -> u64 {
    mix_u64(
        state,
        match overflow {
            TextOverflow::Clip => 1,
            TextOverflow::Ellipsis => 2,
        },
    )
}

fn mix_text_align(state: u64, align: TextAlign) -> u64 {
    mix_u64(
        state,
        match align {
            TextAlign::Start => 1,
            TextAlign::Center => 2,
            TextAlign::End => 3,
        },
    )
}

fn spans_shaping_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut state = 0u64;
    state = mix_u64(state, spans.len() as u64);
    for span in spans {
        state = mix_u64(state, span.len as u64);
        let shaping = &span.shaping;

        state = mix_u64(state, shaping.font.is_some() as u64);
        if let Some(font) = shaping.font.as_ref() {
            state = mix_font_id(state, font);
        }

        state = mix_u64(state, shaping.weight.is_some() as u64);
        if let Some(weight) = shaping.weight {
            state = mix_u64(state, u64::from(weight.0));
        }

        state = mix_u64(state, shaping.slant.is_some() as u64);
        if let Some(slant) = shaping.slant {
            state = mix_text_slant(state, slant);
        }

        state = mix_option_f32(state, shaping.letter_spacing_em);

        state = mix_u64(state, shaping.features.len() as u64);
        for feature in &shaping.features {
            state = mix_bytes(state, feature.tag.as_bytes());
            state = mix_u64(state, u64::from(feature.value));
        }

        state = mix_u64(state, shaping.axes.len() as u64);
        for axis in &shaping.axes {
            state = mix_bytes(state, axis.tag.as_bytes());
            state = mix_f32(state, axis.value);
        }
    }
    state
}
