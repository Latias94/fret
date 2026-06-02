//! Shared chrome resolution helpers for editor controls.
//!
//! v1 goal: keep "frame" defaults consistent across controls (inputs, triggers, scrub surfaces)
//! without hard-binding `fret-ui-editor` to a specific design system crate.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::element::{RingPlacement, RingStyle};
use fret_ui::{TextAreaStyle, TextInputStyle, Theme};
use fret_ui_kit::recipes::input::ResolvedInputChrome;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use super::EditorTokenKeys;
use super::colors::{editor_focus_ring, editor_muted_foreground};

mod surface;

pub(crate) use surface::sanitize_editor_surface_bg;

#[cfg(test)]
mod tests;

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

fn editor_text_field_metric(theme: &Theme, editor_key: &str, legacy_key: &str) -> Option<Px> {
    theme
        .metric_by_key(editor_key)
        .or_else(|| theme.metric_by_key(legacy_key))
}

fn editor_text_field_color(theme: &Theme, editor_key: &str, legacy_key: &str) -> Option<Color> {
    theme
        .color_by_key(editor_key)
        .or_else(|| theme.color_by_key(legacy_key))
}

fn resolve_editor_text_field_input_chrome(
    theme: &Theme,
    size: Size,
    style: &ChromeRefinement,
) -> ResolvedInputChrome {
    let padding_x = editor_text_field_metric(
        theme,
        EditorTokenKeys::TEXT_FIELD_PADDING_X,
        "component.text_field.padding_x",
    )
    .or_else(|| theme.metric_by_key("component.input.padding_x"))
    .unwrap_or_else(|| size.input_px(theme));
    let padding_y = editor_text_field_metric(
        theme,
        EditorTokenKeys::TEXT_FIELD_PADDING_Y,
        "component.text_field.padding_y",
    )
    .or_else(|| theme.metric_by_key("component.input.padding_y"))
    .unwrap_or_else(|| size.input_py(theme));
    let min_height = style
        .min_height
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| {
            editor_text_field_metric(
                theme,
                EditorTokenKeys::TEXT_FIELD_MIN_HEIGHT,
                "component.text_field.min_height",
            )
        })
        .or_else(|| theme.metric_by_key("component.input.min_height"))
        .unwrap_or_else(|| size.input_h(theme));
    let radius = style
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| {
            editor_text_field_metric(
                theme,
                EditorTokenKeys::TEXT_FIELD_RADIUS,
                "component.text_field.radius",
            )
        })
        .or_else(|| theme.metric_by_key("component.input.radius"))
        .unwrap_or_else(|| size.control_radius(theme));
    let border_width = style
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| {
            editor_text_field_metric(
                theme,
                EditorTokenKeys::TEXT_FIELD_BORDER_WIDTH,
                "component.text_field.border_width",
            )
        })
        .or_else(|| theme.metric_by_key("component.input.border_width"))
        .unwrap_or(Px(1.0));

    let background = style
        .background
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| {
            editor_text_field_color(
                theme,
                EditorTokenKeys::TEXT_FIELD_BG,
                "component.text_field.bg",
            )
        })
        .or_else(|| theme.color_by_key("component.input.bg"))
        .unwrap_or_else(|| theme.color_token("background"));
    let border_color = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| {
            editor_text_field_color(
                theme,
                EditorTokenKeys::TEXT_FIELD_BORDER,
                "component.text_field.border",
            )
        })
        .or_else(|| theme.color_by_key("component.input.border"))
        .unwrap_or_else(|| theme.color_token("input"));
    let border_color_focused = editor_text_field_color(
        theme,
        EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS,
        "component.text_field.border_focus",
    )
    .or_else(|| theme.color_by_key("component.input.border_focus"))
    .unwrap_or_else(|| theme.color_token("ring"));
    let text_color = style
        .text_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| {
            editor_text_field_color(
                theme,
                EditorTokenKeys::TEXT_FIELD_FG,
                "component.text_field.fg",
            )
        })
        .or_else(|| theme.color_by_key("component.input.fg"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let text_px = editor_text_field_metric(
        theme,
        EditorTokenKeys::TEXT_FIELD_TEXT_PX,
        "component.text_field.text_px",
    )
    .or_else(|| theme.metric_by_key("component.input.text_px"))
    .unwrap_or_else(|| size.control_text_px(theme));
    let selection_color = editor_text_field_color(
        theme,
        EditorTokenKeys::TEXT_FIELD_SELECTION,
        "component.text_field.selection",
    )
    .or_else(|| theme.color_by_key("component.input.selection"))
    .unwrap_or_else(|| theme.color_token("selection.background"));

    let padding_top = style
        .padding
        .as_ref()
        .and_then(|p| p.top.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_y);
    let padding_bottom = style
        .padding
        .as_ref()
        .and_then(|p| p.bottom.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_y);
    let padding_left = style
        .padding
        .as_ref()
        .and_then(|p| p.left.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_x);
    let padding_right = style
        .padding
        .as_ref()
        .and_then(|p| p.right.as_ref())
        .map(|m| m.resolve(theme))
        .unwrap_or(padding_x);

    ResolvedInputChrome {
        padding: Edges {
            top: Px(padding_top.0.max(0.0)),
            right: Px(padding_right.0.max(0.0)),
            bottom: Px(padding_bottom.0.max(0.0)),
            left: Px(padding_left.0.max(0.0)),
        },
        min_height: Px(min_height.0.max(0.0)),
        radius: Px(radius.0.max(0.0)),
        border_width: Px(border_width.0.max(0.0)),
        background,
        border_color,
        border_color_focused,
        text_color,
        text_px,
        selection_color,
    }
}

pub(crate) fn resolve_editor_text_field_frame_chrome(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> ResolvedEditorFrameChrome {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);
    ResolvedEditorFrameChrome {
        padding: resolved.padding,
        radius: resolved.radius,
        border_width: resolved.border_width,
        bg: sanitize_editor_surface_bg(theme, resolved.background),
        border: resolved.border_color,
        border_focus: resolved.border_color_focused,
        fg: resolved.text_color,
        text_px: resolved.text_px,
    }
}

pub(crate) fn joined_text_input_style(mut chrome: TextInputStyle) -> TextInputStyle {
    chrome.padding = Edges::all(Px(0.0));
    chrome.border = Edges::all(Px(0.0));
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.background = Color {
        a: 0.0,
        ..chrome.background
    };
    chrome.border_color = Color {
        a: 0.0,
        ..chrome.border_color
    };
    chrome.border_color_focused = chrome.border_color;
    chrome.focus_ring = None;
    chrome
}

pub(crate) fn joined_text_area_style(mut chrome: TextAreaStyle) -> TextAreaStyle {
    chrome.padding_x = Px(0.0);
    chrome.padding_y = Px(0.0);
    chrome.border = Edges::all(Px(0.0));
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.background = Color {
        a: 0.0,
        ..chrome.background
    };
    chrome.border_color = Color {
        a: 0.0,
        ..chrome.border_color
    };
    chrome.focus_ring = None;
    chrome
}

pub(crate) fn resolve_editor_text_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextInputStyle, TextStyle) {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = Edges::all(resolved.border_width);
    chrome.background = sanitize_editor_surface_bg(theme, resolved.background);
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.text_color = resolved.text_color;
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = typography::as_control_text(TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    });

    (chrome, text_style)
}

pub(crate) fn resolve_editor_text_area_field_style(
    theme: &Theme,
    size: Size,
    refinement: &ChromeRefinement,
) -> (TextAreaStyle, TextStyle) {
    let resolved = resolve_editor_text_field_input_chrome(theme, size, refinement);
    let ring_color = editor_focus_ring(theme);

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let text_style = typography::as_content_text(TextStyle {
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    });

    let chrome = TextAreaStyle {
        padding_x: resolved.padding.left,
        padding_y: resolved.padding.top,
        background: sanitize_editor_surface_bg(theme, resolved.background),
        border: Edges::all(resolved.border_width),
        border_color: resolved.border_color,
        border_color_focused: resolved.border_color_focused,
        focus_ring: Some(RingStyle {
            placement: RingPlacement::Outset,
            width: Px(2.0),
            offset: Px(2.0),
            color: ring_color,
            offset_color: None,
            corner_radii: Corners::all(resolved.radius),
        }),
        corner_radii: Corners::all(resolved.radius),
        text_color: resolved.text_color,
        placeholder_color: editor_muted_foreground(theme),
        selection_color: resolved.selection_color,
        caret_color: resolved.text_color,
        preedit_bg_color: Color {
            a: 0.22,
            ..resolved.selection_color
        },
        preedit_underline_color: editor_focus_ring(theme),
    };

    (chrome, text_style)
}
