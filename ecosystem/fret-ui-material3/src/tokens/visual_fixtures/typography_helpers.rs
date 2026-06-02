use fret_core::TextStyle;
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

use crate::tokens::typography as token_typography;

use super::token_lookup::{token_number, token_text_style};

pub(super) fn control_text_style(theme: &Theme, key: &str) -> TextStyle {
    typography::with_intent(token_text_style(theme, key), TextIntent::Control)
}

pub(super) fn control_text_style_with_weight(
    theme: &Theme,
    source_key: &str,
    weight_key: &str,
) -> TextStyle {
    text_style_with_weight(theme, source_key, weight_key, TextIntent::Control)
}

pub(super) fn content_text_style_with_weight(
    theme: &Theme,
    source_key: &str,
    weight_key: &str,
) -> TextStyle {
    text_style_with_weight(theme, source_key, weight_key, TextIntent::Content)
}

pub(super) fn text_style_with_weight(
    theme: &Theme,
    source_key: &str,
    weight_key: &str,
    intent: TextIntent,
) -> TextStyle {
    let _ = token_number(theme, weight_key);
    token_typography::text_style_with_weight(theme, None, source_key, Some(weight_key), intent)
}

pub(super) fn text_intent_for_role(role: &str) -> TextIntent {
    if role.contains("action") || role.contains("label") {
        TextIntent::Control
    } else {
        TextIntent::Content
    }
}

pub(super) fn content_text_style(theme: &Theme, key: &str) -> TextStyle {
    typography::with_intent(token_text_style(theme, key), TextIntent::Content)
}
