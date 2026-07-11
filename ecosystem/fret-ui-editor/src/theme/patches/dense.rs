use fret_ui::ThemeConfig;

use crate::primitives::EditorTokenKeys;

use super::{color, metric};

pub(super) fn imgui_like_dense_patch() -> ThemeConfig {
    let mut cfg = ThemeConfig::default();

    // Keep the editor visibly denser while preserving a usable hit target.
    metric(&mut cfg, EditorTokenKeys::DENSITY_ROW_HEIGHT, 22.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_PADDING_X, 5.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_PADDING_Y, 3.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_HIT_THICKNESS, 18.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_ICON_SIZE, 13.0);

    metric(&mut cfg, EditorTokenKeys::NUMERIC_SCRUB_SPEED, 0.035);
    metric(&mut cfg, EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD, 2.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_LABEL_WIDTH, 120.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_COLUMN_GAP, 6.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_TRAILING_GAP, 3.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_ROW_GAP, 4.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_VALUE_MAX_WIDTH, 840.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_STATUS_SLOT_WIDTH, 48.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_RESET_SLOT_WIDTH, 22.0);
    metric(
        &mut cfg,
        EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT,
        24.0,
    );
    metric(&mut cfg, EditorTokenKeys::PROPERTY_GROUP_CONTENT_GAP, 6.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_AUTO_STACK_BELOW, 480.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_GAP, 10.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_HEADER_GAP, 8.0);
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_SIZE, 14.0);
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_RADIUS, 2.0);
    color(&mut cfg, EditorTokenKeys::CHECKBOX_BG, "#1a1c20");
    color(&mut cfg, EditorTokenKeys::CHECKBOX_CHECKED_BG, "#4c88c7");
    color(&mut cfg, EditorTokenKeys::CHECKBOX_CHECKED_FG, "#e6e8eb");
    color(&mut cfg, EditorTokenKeys::CHECKBOX_RING, "#6ea8e0");
    metric(&mut cfg, EditorTokenKeys::VEC_AUTO_STACK_BELOW, 400.0);
    metric(&mut cfg, EditorTokenKeys::VEC_AXIS_MIN_WIDTH, 132.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_TRACK_HEIGHT, 3.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_THUMB_DIAMETER, 10.0);
    color(&mut cfg, EditorTokenKeys::SLIDER_TRACK_BG, "#2a2d33");
    color(&mut cfg, EditorTokenKeys::SLIDER_FILL_BG, "#4c88c7");
    color(&mut cfg, EditorTokenKeys::SLIDER_THUMB_BG, "#1a1c20");
    color(&mut cfg, EditorTokenKeys::SLIDER_THUMB_BORDER, "#4b5563");

    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_PADDING_X, 5.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_PADDING_Y, 3.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_MIN_HEIGHT, 22.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_RADIUS, 2.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_BORDER_WIDTH, 1.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_TEXT_PX, 12.0);

    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_BG, "#1a1c20");
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_BORDER, "#4b5563");
    color(
        &mut cfg,
        EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS,
        "#6ea8e0",
    );
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_FG, "#e6e8eb");
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_SELECTION, "#315b8b");
    color(&mut cfg, EditorTokenKeys::CHROME_MUTED_FG, "#acb4bf");
    color(&mut cfg, EditorTokenKeys::CHROME_ACCENT, "#4c88c7");
    color(&mut cfg, EditorTokenKeys::CHROME_RING, "#6ea8e0");

    color(&mut cfg, "card", "#202328");
    color(&mut cfg, "background", "#171a1f");
    color(&mut cfg, "muted", "#2a2d33");
    color(&mut cfg, "secondary", "#2c3138");
    color(&mut cfg, "secondary-foreground", "#e6e8eb");
    color(&mut cfg, "primary", "#4c88c7");
    color(&mut cfg, "primary-foreground", "#e6e8eb");
    color(&mut cfg, "border", "#454d59");
    color(&mut cfg, "input", "#4b5563");
    color(&mut cfg, "foreground", "#e6e8eb");
    color(&mut cfg, "muted-foreground", "#acb4bf");
    color(&mut cfg, "accent", "#4c88c7");
    color(&mut cfg, "accent-foreground", "#e6e8eb");
    color(&mut cfg, "ring", "#6ea8e0");
    color(&mut cfg, "popover", "#24292f");
    color(&mut cfg, "popover-foreground", "#e6e8eb");
    color(&mut cfg, "selection.background", "#315b8b");
    color(&mut cfg, EditorTokenKeys::POPUP_BG, "#24292f");
    color(&mut cfg, EditorTokenKeys::POPUP_BORDER, "#687686");
    color(&mut cfg, EditorTokenKeys::POPUP_SHADOW_COLOR, "#2a2d33");
    metric(&mut cfg, EditorTokenKeys::POPUP_RADIUS, 4.0);
    metric(&mut cfg, EditorTokenKeys::POPUP_SHADOW_OFFSET_Y, 4.0);
    metric(&mut cfg, EditorTokenKeys::POPUP_SHADOW_BLUR, 12.0);
    metric(&mut cfg, EditorTokenKeys::POPUP_SHADOW_SPREAD, -3.0);

    color(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_BG, "#1d2127");
    color(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_BORDER, "#54606d");
    color(
        &mut cfg,
        EditorTokenKeys::PROPERTY_PANEL_HEADER_BG,
        "#36414c",
    );
    color(
        &mut cfg,
        EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER,
        "#728294",
    );
    metric(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_RADIUS, 2.0);
    color(&mut cfg, EditorTokenKeys::PROPERTY_GROUP_BORDER, "#47515d");
    color(&mut cfg, EditorTokenKeys::PROPERTY_HEADER_BG, "#283039");
    color(&mut cfg, EditorTokenKeys::PROPERTY_HEADER_BORDER, "#56626f");
    color(&mut cfg, EditorTokenKeys::PROPERTY_HEADER_FG, "#e6e8eb");

    color(&mut cfg, EditorTokenKeys::CONTROL_INVALID_FG, "#ffcbc7");
    color(&mut cfg, EditorTokenKeys::CONTROL_INVALID_BORDER, "#d06a6a");
    color(&mut cfg, EditorTokenKeys::CONTROL_INVALID_BG, "#362225");
    color(&mut cfg, EditorTokenKeys::NUMERIC_ERROR_FG, "#ffcbc7");
    color(&mut cfg, EditorTokenKeys::NUMERIC_ERROR_BORDER, "#d06a6a");
    color(&mut cfg, EditorTokenKeys::NUMERIC_ERROR_BG, "#362225");

    cfg
}
