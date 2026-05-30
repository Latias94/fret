use fret_ui::ThemeConfig;

use crate::primitives::EditorTokenKeys;

use super::EditorThemePresetV1;

pub(super) fn editor_theme_patch_v1() -> ThemeConfig {
    let mut cfg = ThemeConfig::default();

    // Editor density defaults (used by most controls).
    metric(&mut cfg, EditorTokenKeys::DENSITY_ROW_HEIGHT, 24.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_PADDING_X, 6.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_PADDING_Y, 4.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_HIT_THICKNESS, 20.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_ICON_SIZE, 14.0);

    // Checkbox metrics and colors (used by TransformEdit link toggles and inspector rows).
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_SIZE, 16.0);
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_RADIUS, 4.0);
    color(&mut cfg, EditorTokenKeys::CHECKBOX_BG, "#141b24");
    color(&mut cfg, EditorTokenKeys::CHECKBOX_CHECKED_BG, "#355a86");
    color(&mut cfg, EditorTokenKeys::CHECKBOX_CHECKED_FG, "#edf3fa");
    color(&mut cfg, EditorTokenKeys::CHECKBOX_RING, "#7faee8");

    // Vec edit responsiveness (stack axes vertically in narrow inspectors).
    metric(&mut cfg, EditorTokenKeys::VEC_AUTO_STACK_BELOW, 420.0);
    metric(&mut cfg, EditorTokenKeys::VEC_AXIS_MIN_WIDTH, 140.0);

    // Property grid responsiveness (stack label/value vertically in narrow inspectors).
    metric(&mut cfg, EditorTokenKeys::PROPERTY_LABEL_WIDTH, 124.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_COLUMN_GAP, 10.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_TRAILING_GAP, 6.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_ROW_GAP, 5.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_VALUE_MAX_WIDTH, 1024.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_STATUS_SLOT_WIDTH, 56.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_RESET_SLOT_WIDTH, 24.0);
    metric(
        &mut cfg,
        EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT,
        28.0,
    );
    metric(&mut cfg, EditorTokenKeys::PROPERTY_GROUP_CONTENT_GAP, 10.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_AUTO_STACK_BELOW, 520.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_GAP, 14.0);
    metric(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_HEADER_GAP, 12.0);

    // Editor-owned text-field-like metrics (used by MiniSearchBox / NumericInput / ColorEdit).
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_PADDING_X, 6.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_PADDING_Y, 4.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_MIN_HEIGHT, 24.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_RADIUS, 4.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_BORDER_WIDTH, 1.0);
    metric(&mut cfg, EditorTokenKeys::TEXT_FIELD_TEXT_PX, 12.0);

    // Default editor baseline colors. These stay more technical and contrast-forward than the
    // underlying app theme without turning the default preset into an imgui clone.
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_BG, "#141b24");
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_BORDER, "#3b4758");
    color(
        &mut cfg,
        EditorTokenKeys::TEXT_FIELD_BORDER_FOCUS,
        "#7faee8",
    );
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_FG, "#edf3fa");
    color(&mut cfg, EditorTokenKeys::TEXT_FIELD_SELECTION, "#284d75");
    color(&mut cfg, EditorTokenKeys::CHROME_MUTED_FG, "#9eabbc");
    color(&mut cfg, EditorTokenKeys::CHROME_ACCENT, "#355a86");
    color(&mut cfg, EditorTokenKeys::CHROME_RING, "#7faee8");

    color(&mut cfg, "card", "#10161e");
    color(&mut cfg, "background", "#0c1118");
    color(&mut cfg, "muted", "#171d26");
    color(&mut cfg, "secondary", "#1a2230");
    color(&mut cfg, "secondary-foreground", "#edf3fa");
    color(&mut cfg, "primary", "#355a86");
    color(&mut cfg, "primary-foreground", "#edf3fa");
    color(&mut cfg, "border", "#2f3a48");
    color(&mut cfg, "input", "#3b4758");
    color(&mut cfg, "foreground", "#edf3fa");
    color(&mut cfg, "muted-foreground", "#9eabbc");
    color(&mut cfg, "accent", "#355a86");
    color(&mut cfg, "accent-foreground", "#edf3fa");
    color(&mut cfg, "ring", "#7faee8");
    color(&mut cfg, "popover", "#131b25");
    color(&mut cfg, "popover-foreground", "#edf3fa");
    color(&mut cfg, "selection.background", "#315b8b");
    color(&mut cfg, EditorTokenKeys::POPUP_BG, "#131b25");
    color(&mut cfg, EditorTokenKeys::POPUP_BORDER, "#46596c");
    color(&mut cfg, EditorTokenKeys::POPUP_SHADOW_COLOR, "#171d26");
    metric(&mut cfg, EditorTokenKeys::POPUP_RADIUS, 8.0);
    metric(&mut cfg, EditorTokenKeys::POPUP_SHADOW_OFFSET_Y, 6.0);
    metric(&mut cfg, EditorTokenKeys::POPUP_SHADOW_BLUR, 16.0);
    metric(&mut cfg, EditorTokenKeys::POPUP_SHADOW_SPREAD, -4.0);

    color(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_BG, "#0f151d");
    color(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_BORDER, "#3d4d5f");
    color(
        &mut cfg,
        EditorTokenKeys::PROPERTY_PANEL_HEADER_BG,
        "#243445",
    );
    color(
        &mut cfg,
        EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER,
        "#5a7087",
    );
    metric(&mut cfg, EditorTokenKeys::PROPERTY_PANEL_RADIUS, 6.0);
    color(&mut cfg, EditorTokenKeys::PROPERTY_GROUP_BORDER, "#33414f");
    color(&mut cfg, EditorTokenKeys::PROPERTY_HEADER_BG, "#19232e");
    color(&mut cfg, EditorTokenKeys::PROPERTY_HEADER_BORDER, "#384857");
    color(&mut cfg, EditorTokenKeys::PROPERTY_HEADER_FG, "#edf3fa");

    color(&mut cfg, EditorTokenKeys::CONTROL_INVALID_FG, "#ffd3d6");
    color(&mut cfg, EditorTokenKeys::CONTROL_INVALID_BORDER, "#c76f77");
    color(&mut cfg, EditorTokenKeys::CONTROL_INVALID_BG, "#2a171c");
    color(&mut cfg, EditorTokenKeys::NUMERIC_ERROR_FG, "#ffd3d6");
    color(&mut cfg, EditorTokenKeys::NUMERIC_ERROR_BORDER, "#c76f77");
    color(&mut cfg, EditorTokenKeys::NUMERIC_ERROR_BG, "#2a171c");

    // Numeric scrub defaults. These are included in the base patch so switching back from a dense
    // preset clears dense-only scrub overrides instead of leaving stale token values behind.
    metric(&mut cfg, EditorTokenKeys::NUMERIC_SCRUB_SPEED, 0.02);
    metric(
        &mut cfg,
        EditorTokenKeys::NUMERIC_SCRUB_SLOW_MULTIPLIER,
        0.1,
    );
    metric(
        &mut cfg,
        EditorTokenKeys::NUMERIC_SCRUB_FAST_MULTIPLIER,
        10.0,
    );
    metric(&mut cfg, EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD, 4.0);

    // Slider metrics and colors (normalized floats like roughness/metallic).
    metric(&mut cfg, EditorTokenKeys::SLIDER_TRACK_HEIGHT, 4.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_THUMB_DIAMETER, 12.0);
    color(&mut cfg, EditorTokenKeys::SLIDER_TRACK_BG, "#171d26");
    color(&mut cfg, EditorTokenKeys::SLIDER_FILL_BG, "#355a86");
    color(&mut cfg, EditorTokenKeys::SLIDER_THUMB_BG, "#141b24");
    color(&mut cfg, EditorTokenKeys::SLIDER_THUMB_BORDER, "#3b4758");

    cfg
}

pub(super) fn editor_theme_preset_overrides_v1(preset: EditorThemePresetV1) -> Option<ThemeConfig> {
    match preset {
        EditorThemePresetV1::Default => None,
        EditorThemePresetV1::ImguiLikeDense => Some(imgui_like_dense_patch_v1()),
    }
}

fn imgui_like_dense_patch_v1() -> ThemeConfig {
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

fn metric(cfg: &mut ThemeConfig, key: &str, value: f32) {
    cfg.metrics.insert(key.to_string(), value);
}

fn color(cfg: &mut ThemeConfig, key: &str, value: &str) {
    cfg.colors.insert(key.to_string(), value.to_string());
}
