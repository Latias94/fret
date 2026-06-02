//! Shared editor semantic color helpers.
//!
//! These helpers keep editor-owned surfaces on the editor token lane first while preserving
//! generic app-theme fallback for compatibility.

use fret_core::Color;
use fret_ui::Theme;

use super::EditorTokenKeys;

#[cfg(test)]
mod tests;

pub(crate) fn editor_foreground(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::TEXT_FIELD_FG)
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

pub(crate) fn editor_muted_foreground(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CHROME_MUTED_FG)
        .or_else(|| theme.color_by_key("muted-foreground"))
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| editor_foreground(theme))
}

pub(crate) fn editor_accent(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CHROME_ACCENT)
        .unwrap_or_else(|| theme.color_token("accent"))
}

pub(crate) fn editor_focus_ring(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CHROME_RING)
        .or_else(|| theme.color_by_key(EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS))
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_token("primary"))
}

pub(crate) fn editor_invalid_foreground(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CONTROL_INVALID_FG)
        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG))
        .unwrap_or_else(|| theme.color_token("destructive"))
}

pub(crate) fn editor_invalid_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::CONTROL_INVALID_BORDER)
        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_BORDER))
        .or_else(|| theme.color_by_key(EditorTokenKeys::CONTROL_INVALID_FG))
        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG))
        .unwrap_or_else(|| theme.color_token("destructive"))
}

pub(crate) fn editor_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::TEXT_FIELD_BORDER)
        .or_else(|| theme.color_by_key("component.text_field.border"))
        .or_else(|| theme.color_by_key("component.input.border"))
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"))
}

pub(crate) fn editor_panel_background(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG)
        .or_else(|| theme.color_by_key("card"))
        .or_else(|| theme.color_by_key("component.card.bg"))
        .unwrap_or_else(|| theme.color_token("background"))
}

pub(crate) fn editor_panel_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_PANEL_BORDER)
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("component.card.border"))
        .unwrap_or_else(|| editor_border(theme))
}

pub(crate) fn editor_panel_header_background(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BG)
        .or_else(|| theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG))
        .or_else(|| theme.color_by_key("muted"))
        .or_else(|| theme.color_by_key("component.card.bg"))
        .unwrap_or_else(|| editor_panel_background(theme))
}

pub(crate) fn editor_panel_header_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER)
        .or_else(|| theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER))
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("component.card.border"))
        .unwrap_or_else(|| editor_panel_border(theme))
}

pub(crate) fn editor_property_group_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_GROUP_BORDER)
        .or_else(|| theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_BORDER))
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("component.card.border"))
        .unwrap_or_else(|| editor_panel_border(theme))
}

pub(crate) fn editor_property_header_background(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG)
        .or_else(|| theme.color_by_key("muted"))
        .or_else(|| theme.color_by_key("component.card.bg"))
        .unwrap_or_else(|| editor_panel_background(theme))
}

pub(crate) fn editor_property_header_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER)
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("component.card.border"))
        .unwrap_or_else(|| editor_panel_border(theme))
}

pub(crate) fn editor_property_header_foreground(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::PROPERTY_HEADER_FG)
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| editor_foreground(theme))
}

pub(crate) fn editor_popup_background(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::POPUP_BG)
        .or_else(|| theme.color_by_key("component.text_field.bg"))
        .or_else(|| theme.color_by_key("popover"))
        .unwrap_or_else(|| editor_panel_background(theme))
}

pub(crate) fn editor_popup_border(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::POPUP_BORDER)
        .or_else(|| theme.color_by_key("component.text_field.border"))
        .unwrap_or_else(|| editor_panel_border(theme))
}

pub(crate) fn editor_subtle_bg(theme: &Theme) -> Color {
    theme
        .color_by_key(EditorTokenKeys::TEXT_FIELD_BG)
        .or_else(|| theme.color_by_key("component.text_field.bg"))
        .or_else(|| theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG))
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"))
}
