use crate::ThemeSnapshot;
use fret_core::{
    AttributedText, FontId, Px, TextAlign, TextInput, TextOverflow, TextSlant, TextSpan, TextStyle,
    TextWrap,
};

pub(crate) fn default_text_style(theme: ThemeSnapshot) -> TextStyle {
    let size = theme
        .metric_by_key("font.size")
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

pub(crate) fn resolve_text_style(theme: ThemeSnapshot, explicit: Option<TextStyle>) -> TextStyle {
    let mut style = explicit.unwrap_or_else(|| default_text_style(theme.clone()));
    if style.line_height.is_none() {
        style.line_height = Some(derive_default_line_height(theme, &style));
    }
    style
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
            TextWrap::WordBreak => 4,
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

fn derive_default_line_height(theme: ThemeSnapshot, style: &TextStyle) -> Px {
    let (base_size, base_line_height) = match style.font {
        FontId::Monospace => (
            theme
                .metric_by_key("mono_font.size")
                .unwrap_or(theme.metrics.mono_font_size),
            theme
                .metric_by_key("mono_font.line_height")
                .unwrap_or(theme.metrics.mono_font_line_height),
        ),
        _ => (
            theme
                .metric_by_key("font.size")
                .unwrap_or(theme.metrics.font_size),
            theme
                .metric_by_key("font.line_height")
                .unwrap_or(theme.metrics.font_line_height),
        ),
    };

    let base_size_px = base_size.0;
    let base_line_height_px = base_line_height.0;
    let ratio = if base_size_px.is_finite()
        && base_line_height_px.is_finite()
        && base_size_px > 0.0
        && base_line_height_px > 0.0
    {
        base_line_height_px / base_size_px
    } else {
        1.25
    };

    let size_px = style.size.0.max(0.0);
    Px((size_px * ratio).max(size_px))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeColors, ThemeMetrics};
    use fret_core::Color;

    fn dummy_theme_snapshot() -> ThemeSnapshot {
        let metrics = ThemeMetrics {
            radius_sm: Px(0.0),
            radius_md: Px(0.0),
            radius_lg: Px(0.0),
            padding_sm: Px(0.0),
            padding_md: Px(0.0),
            scrollbar_width: Px(0.0),
            font_size: Px(13.0),
            mono_font_size: Px(13.0),
            font_line_height: Px(16.0),
            mono_font_line_height: Px(16.0),
        };
        let c = Color::TRANSPARENT;
        let colors = ThemeColors {
            surface_background: c,
            panel_background: c,
            panel_border: c,
            text_primary: c,
            text_muted: c,
            text_disabled: c,
            accent: c,
            selection_background: c,
            selection_inactive_background: c,
            selection_window_inactive_background: c,
            hover_background: c,
            focus_ring: c,
            menu_background: c,
            menu_border: c,
            menu_item_hover: c,
            menu_item_selected: c,
            list_background: c,
            list_border: c,
            list_row_hover: c,
            list_row_selected: c,
            scrollbar_track: c,
            scrollbar_thumb: c,
            scrollbar_thumb_hover: c,
            viewport_selection_fill: c,
            viewport_selection_stroke: c,
            viewport_marker: c,
            viewport_drag_line_pan: c,
            viewport_drag_line_orbit: c,
            viewport_gizmo_x: c,
            viewport_gizmo_y: c,
            viewport_gizmo_handle_background: c,
            viewport_gizmo_handle_border: c,
            viewport_rotate_gizmo: c,
        };

        ThemeSnapshot::from_baseline(colors, metrics, 0)
    }

    #[test]
    fn resolves_missing_line_height_from_theme_ratio() {
        let theme = dummy_theme_snapshot();
        let style = TextStyle {
            size: Px(10.0),
            line_height: None,
            ..Default::default()
        };

        let resolved = resolve_text_style(theme, Some(style));
        let got = resolved.line_height.unwrap().0;
        let expected = 10.0_f32 * (16.0_f32 / 13.0_f32);
        assert!(
            (got - expected).abs() < 1e-4,
            "got={got} expected={expected}"
        );
    }
}
