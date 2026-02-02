use crate::ThemeSnapshot;
use fret_core::{TextInput, TextStyle};

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
