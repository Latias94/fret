//! Shared chrome resolution helpers for editor controls.
//!
//! v1 goal: keep "frame" defaults consistent across controls (inputs, triggers, scrub surfaces)
//! without hard-binding `fret-ui-editor` to a specific design system crate.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::{TextInputStyle, Theme};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, Size};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEditorFrameChrome {
    pub(crate) padding: Edges,
    pub(crate) radius: Px,
    pub(crate) border_width: Px,
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) border_focus: Color,
    pub(crate) fg: Color,
    pub(crate) text_px: Px,
}

pub(crate) fn resolve_editor_frame_chrome(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
    keys: InputTokenKeys,
) -> ResolvedEditorFrameChrome {
    let resolved = resolve_input_chrome(theme, size, refinement, keys);
    ResolvedEditorFrameChrome {
        padding: resolved.padding,
        radius: resolved.radius,
        border_width: resolved.border_width,
        bg: resolved.background,
        border: resolved.border_color,
        border_focus: resolved.border_color_focused,
        fg: resolved.text_color,
        text_px: resolved.text_px,
    }
}

pub(crate) fn resolve_editor_text_input_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
    keys: InputTokenKeys,
) -> (TextInputStyle, TextStyle) {
    let resolved = resolve_input_chrome(theme, size, refinement, keys);

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = Edges::all(resolved.border_width);
    chrome.background = resolved.background;
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.text_color = resolved.text_color;
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    };

    (chrome, text_style)
}

pub(crate) fn resolve_editor_text_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextInputStyle, TextStyle) {
    resolve_editor_text_input_style(
        theme,
        size,
        refinement,
        InputTokenKeys {
            padding_x: Some("component.text_field.padding_x"),
            padding_y: Some("component.text_field.padding_y"),
            min_height: Some("component.text_field.min_height"),
            radius: Some("component.text_field.radius"),
            border_width: Some("component.text_field.border_width"),
            bg: Some("component.text_field.bg"),
            border: Some("component.text_field.border"),
            border_focus: Some("component.text_field.border_focus"),
            fg: Some("component.text_field.fg"),
            text_px: Some("component.text_field.text_px"),
            selection: Some("component.text_field.selection"),
        },
    )
}
