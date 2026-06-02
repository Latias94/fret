use fret_core::{Color, Edges, Px};
use fret_ui::Theme;
use fret_ui_kit::recipes::input::ResolvedInputChrome;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::EditorTokenKeys;

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

pub(super) fn resolve_editor_text_field_input_chrome(
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
