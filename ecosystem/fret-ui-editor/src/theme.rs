//! Editor-oriented theme patch helpers.
//!
//! These helpers are intentionally opt-in. They should be used by demos/apps that want an
//! editor-like density baseline without depending on a full design-system crate.

use fret_ui::{Theme, ThemeConfig, UiHost};

use crate::primitives::EditorTokenKeys;

/// Apply a small set of editor-oriented metric overrides.
///
/// This is designed as a patch layered on top of an existing theme (e.g. shadcn New York) and is
/// safe to call multiple times.
pub fn apply_editor_theme_patch_v1<H: UiHost>(app: &mut H) {
    Theme::with_global_mut(app, |theme| {
        let mut cfg = ThemeConfig::default();

        // Editor density defaults (used by most controls).
        cfg.metrics
            .insert(EditorTokenKeys::DENSITY_ROW_HEIGHT.to_string(), 24.0);
        cfg.metrics
            .insert(EditorTokenKeys::DENSITY_PADDING_X.to_string(), 6.0);
        cfg.metrics
            .insert(EditorTokenKeys::DENSITY_PADDING_Y.to_string(), 4.0);
        cfg.metrics
            .insert(EditorTokenKeys::DENSITY_HIT_THICKNESS.to_string(), 20.0);
        cfg.metrics
            .insert(EditorTokenKeys::DENSITY_ICON_SIZE.to_string(), 14.0);

        // Checkbox metrics (used by TransformEdit link toggles and inspector rows).
        cfg.metrics
            .insert(EditorTokenKeys::CHECKBOX_SIZE.to_string(), 16.0);
        cfg.metrics
            .insert(EditorTokenKeys::CHECKBOX_RADIUS.to_string(), 4.0);

        // Text-field-like metrics (used by MiniSearchBox / NumericInput / ColorEdit).
        cfg.metrics
            .insert("component.text_field.padding_x".to_string(), 6.0);
        cfg.metrics
            .insert("component.text_field.padding_y".to_string(), 4.0);
        cfg.metrics
            .insert("component.text_field.min_height".to_string(), 24.0);
        cfg.metrics
            .insert("component.text_field.radius".to_string(), 4.0);
        cfg.metrics
            .insert("component.text_field.border_width".to_string(), 1.0);
        cfg.metrics
            .insert("component.text_field.text_px".to_string(), 12.0);

        theme.apply_config_patch(&cfg);
    });
}
