//! Shared helpers for component typography token access.

use fret_core::{FontWeight, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

pub(crate) fn text_style(
    theme: &Theme,
    primary_key: Option<&str>,
    source_key: &str,
    intent: TextIntent,
) -> TextStyle {
    let style = primary_key
        .and_then(|key| theme.text_style_by_key(key))
        .or_else(|| theme.text_style_by_key(source_key))
        .unwrap_or_default();

    typography::with_intent(style, intent)
}

pub(crate) fn text_style_with_weight(
    theme: &Theme,
    primary_key: Option<&str>,
    source_key: &str,
    weight_key: Option<&str>,
    intent: TextIntent,
) -> TextStyle {
    let mut style = text_style(theme, primary_key, source_key, intent);

    if let Some(weight) = weight_key.and_then(|key| theme.number_by_key(key)) {
        style.weight = FontWeight(weight.round().clamp(1.0, 1000.0) as u16);
    }

    style
}

pub(crate) fn text_style_with_weight_fallback(
    theme: &Theme,
    primary_key: Option<&str>,
    source_key: &str,
    weight_key: &str,
    fallback_weight: f32,
    intent: TextIntent,
) -> TextStyle {
    let mut style = text_style(theme, primary_key, source_key, intent);
    let weight = theme.number_by_key(weight_key).unwrap_or(fallback_weight);
    style.weight = FontWeight(weight.round().clamp(1.0, 1000.0) as u16);
    style
}
