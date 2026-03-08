use crate::ThemeSnapshot;
use fret_core::{
    AttributedText, FontId, Px, TextAlign, TextInput, TextLineHeightPolicy, TextOverflow,
    TextSlant, TextSpan, TextStyle, TextStyleRefinement, TextWrap,
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
        line_height_policy: TextLineHeightPolicy::FixedFromStyle,
        ..Default::default()
    }
}

pub(crate) fn resolve_text_style(
    theme: ThemeSnapshot,
    explicit: Option<TextStyle>,
    inherited: Option<&TextStyleRefinement>,
) -> TextStyle {
    let mut style = if let Some(explicit) = explicit {
        explicit
    } else {
        let mut style = default_text_style(theme.clone());
        if let Some(inherited) = inherited {
            style.refine(inherited);
        }
        style
    };
    if style.line_height.is_none() && style.line_height_em.is_none() {
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
    resolved_style: &TextStyle,
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
    state = mix_f32(state, scale_factor);
    state = mix_text_wrap(state, TextWrap::None);
    state = mix_text_overflow(state, overflow);
    state = mix_text_align(state, align);
    state = mix_text_style(state, resolved_style);
    state
}

pub(crate) fn text_wrap_none_measure_fingerprint_rich(
    rich: &AttributedText,
    resolved_style: &TextStyle,
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
    state = mix_f32(state, scale_factor);
    state = mix_text_wrap(state, TextWrap::None);
    state = mix_text_overflow(state, overflow);
    state = mix_text_align(state, align);
    state = mix_text_style(state, resolved_style);
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

pub(crate) fn text_style_refinement_fingerprint(refinement: &TextStyleRefinement) -> u64 {
    let mut state = 0u64;
    state = mix_u64(state, 3);

    state = mix_u64(state, refinement.font.is_some() as u64);
    if let Some(font) = refinement.font.as_ref() {
        state = mix_font_id(state, font);
    }

    state = mix_u64(state, refinement.size.is_some() as u64);
    if let Some(size) = refinement.size {
        state = mix_px(state, size);
    }

    state = mix_u64(state, refinement.weight.is_some() as u64);
    if let Some(weight) = refinement.weight {
        state = mix_u64(state, u64::from(weight.0));
    }

    state = mix_u64(state, refinement.slant.is_some() as u64);
    if let Some(slant) = refinement.slant {
        state = mix_text_slant(state, slant);
    }

    state = mix_option_px(state, refinement.line_height);
    state = mix_option_f32(state, refinement.line_height_em);

    state = mix_u64(state, refinement.line_height_policy.is_some() as u64);
    if let Some(policy) = refinement.line_height_policy {
        state = mix_u64(
            state,
            match policy {
                fret_core::TextLineHeightPolicy::ExpandToFit => 1,
                fret_core::TextLineHeightPolicy::FixedFromStyle => 2,
            },
        );
    }

    state = mix_option_f32(state, refinement.letter_spacing_em);

    state = mix_u64(state, refinement.vertical_placement.is_some() as u64);
    if let Some(vertical_placement) = refinement.vertical_placement {
        state = mix_u64(
            state,
            match vertical_placement {
                fret_core::TextVerticalPlacement::CenterMetricsBox => 1,
                fret_core::TextVerticalPlacement::BoundsAsLineBox => 2,
            },
        );
    }

    state = mix_u64(state, refinement.leading_distribution.is_some() as u64);
    if let Some(leading_distribution) = refinement.leading_distribution {
        state = mix_u64(
            state,
            match leading_distribution {
                fret_core::TextLeadingDistribution::Even => 1,
                fret_core::TextLeadingDistribution::Proportional => 2,
            },
        );
    }

    state
}

fn mix_text_style(mut state: u64, style: &TextStyle) -> u64 {
    state = mix_font_id(state, &style.font);
    state = mix_px(state, style.size);
    state = mix_u64(state, u64::from(style.weight.0));
    state = mix_text_slant(state, style.slant);
    state = mix_option_px(state, style.line_height);
    state = mix_option_f32(state, style.line_height_em);
    state = mix_u64(
        state,
        match style.line_height_policy {
            fret_core::TextLineHeightPolicy::ExpandToFit => 1,
            fret_core::TextLineHeightPolicy::FixedFromStyle => 2,
        },
    );
    state = mix_option_f32(state, style.letter_spacing_em);
    state = mix_u64(
        state,
        match style.vertical_placement {
            fret_core::TextVerticalPlacement::CenterMetricsBox => 1,
            fret_core::TextVerticalPlacement::BoundsAsLineBox => 2,
        },
    );
    state = mix_u64(
        state,
        match style.leading_distribution {
            fret_core::TextLeadingDistribution::Even => 1,
            fret_core::TextLeadingDistribution::Proportional => 2,
        },
    );
    state = mix_u64(state, style.features.len() as u64);
    for feature in &style.features {
        state = mix_bytes(state, feature.tag.as_bytes());
        state = mix_u64(state, u64::from(feature.value));
    }
    state = mix_u64(state, style.axes.len() as u64);
    for axis in &style.axes {
        state = mix_bytes(state, axis.tag.as_bytes());
        state = mix_f32(state, axis.value);
    }
    state = mix_u64(state, style.strut_style.is_some() as u64);
    if let Some(strut) = style.strut_style.as_ref() {
        state = mix_u64(state, strut.font.is_some() as u64);
        if let Some(font) = strut.font.as_ref() {
            state = mix_font_id(state, font);
        }
        state = mix_option_px(state, strut.size);
        state = mix_option_px(state, strut.line_height);
        state = mix_option_f32(state, strut.line_height_em);
        state = mix_u64(
            state,
            match strut.leading_distribution.unwrap_or_default() {
                fret_core::TextLeadingDistribution::Even => 1,
                fret_core::TextLeadingDistribution::Proportional => 2,
            },
        );
        state = mix_u64(state, u64::from(strut.force));
    }
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
            TextWrap::Balance => 3,
            TextWrap::Grapheme => 4,
            TextWrap::WordBreak => 5,
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

        let resolved = resolve_text_style(theme, Some(style), None);
        let got = resolved.line_height.unwrap().0;
        let expected = 10.0_f32 * (16.0_f32 / 13.0_f32);
        assert!(
            (got - expected).abs() < 1e-4,
            "got={got} expected={expected}"
        );
    }
}
