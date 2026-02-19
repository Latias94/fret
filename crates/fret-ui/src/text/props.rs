use crate::ThemeSnapshot;
use fret_core::{FontId, Px, TextInput, TextStyle};

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

