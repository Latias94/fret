//! Editor-oriented theme patch helpers.
//!
//! These helpers are intentionally opt-in. They should be used by demos/apps that want an
//! editor-like density baseline without depending on a full design-system crate.

use fret_ui::{Theme, ThemeConfig, UiHost};

use crate::primitives::EditorTokenKeys;

/// Named editor presets layered on top of an app-selected base theme.
///
/// These presets intentionally stay in the policy layer: they patch existing theme tokens instead
/// of creating a second widget tree or a new runtime-level theme namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorThemePresetV1 {
    /// Conservative editor density baseline intended to preserve current demo behavior.
    #[default]
    Default,
    /// Dense, square-ish editor chrome inspired by imgui-class tooling.
    ImguiLikeDense,
}

impl EditorThemePresetV1 {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::ImguiLikeDense => "imgui_like_dense",
        }
    }
}

/// Apply an editor-oriented preset layered on top of the current theme.
///
/// This is designed as a patch on top of an existing theme (e.g. shadcn New York) and is safe to
/// call multiple times.
pub fn apply_editor_theme_preset_v1<H: UiHost>(app: &mut H, preset: EditorThemePresetV1) {
    Theme::with_global_mut(app, |theme| {
        theme.apply_config_patch(&editor_theme_patch_v1());

        if let Some(preset_cfg) = editor_theme_preset_overrides_v1(preset) {
            theme.apply_config_patch(&preset_cfg);
        }
    });
}

/// Apply the default editor density patch.
///
/// This remains as the compatibility wrapper for older callsites.
pub fn apply_editor_theme_patch_v1<H: UiHost>(app: &mut H) {
    apply_editor_theme_preset_v1(app, EditorThemePresetV1::Default);
}

fn editor_theme_patch_v1() -> ThemeConfig {
    let mut cfg = ThemeConfig::default();

    // Editor density defaults (used by most controls).
    metric(&mut cfg, EditorTokenKeys::DENSITY_ROW_HEIGHT, 24.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_PADDING_X, 6.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_PADDING_Y, 4.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_HIT_THICKNESS, 20.0);
    metric(&mut cfg, EditorTokenKeys::DENSITY_ICON_SIZE, 14.0);

    // Checkbox metrics (used by TransformEdit link toggles and inspector rows).
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_SIZE, 16.0);
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_RADIUS, 4.0);

    // Vec edit responsiveness (stack axes vertically in narrow inspectors).
    metric(&mut cfg, EditorTokenKeys::VEC_AUTO_STACK_BELOW, 420.0);
    metric(&mut cfg, EditorTokenKeys::VEC_AXIS_MIN_WIDTH, 140.0);

    // Property grid responsiveness (stack label/value vertically in narrow inspectors).
    metric(&mut cfg, EditorTokenKeys::PROPERTY_AUTO_STACK_BELOW, 520.0);

    // Text-field-like metrics (used by MiniSearchBox / NumericInput / ColorEdit).
    metric(&mut cfg, "component.text_field.padding_x", 6.0);
    metric(&mut cfg, "component.text_field.padding_y", 4.0);
    metric(&mut cfg, "component.text_field.min_height", 24.0);
    metric(&mut cfg, "component.text_field.radius", 4.0);
    metric(&mut cfg, "component.text_field.border_width", 1.0);
    metric(&mut cfg, "component.text_field.text_px", 12.0);

    // Slider metrics (normalized floats like roughness/metallic).
    metric(&mut cfg, EditorTokenKeys::SLIDER_TRACK_HEIGHT, 4.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_THUMB_DIAMETER, 12.0);

    cfg
}

fn editor_theme_preset_overrides_v1(preset: EditorThemePresetV1) -> Option<ThemeConfig> {
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
    metric(&mut cfg, EditorTokenKeys::PROPERTY_COLUMN_GAP, 6.0);
    metric(
        &mut cfg,
        EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT,
        22.0,
    );
    metric(&mut cfg, EditorTokenKeys::PROPERTY_AUTO_STACK_BELOW, 480.0);
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_SIZE, 14.0);
    metric(&mut cfg, EditorTokenKeys::CHECKBOX_RADIUS, 2.0);
    metric(&mut cfg, EditorTokenKeys::VEC_AUTO_STACK_BELOW, 400.0);
    metric(&mut cfg, EditorTokenKeys::VEC_AXIS_MIN_WIDTH, 132.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_TRACK_HEIGHT, 3.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_THUMB_DIAMETER, 10.0);

    metric(&mut cfg, "component.text_field.padding_x", 5.0);
    metric(&mut cfg, "component.text_field.padding_y", 3.0);
    metric(&mut cfg, "component.text_field.min_height", 22.0);
    metric(&mut cfg, "component.text_field.radius", 2.0);
    metric(&mut cfg, "component.text_field.border_width", 1.0);
    metric(&mut cfg, "component.text_field.text_px", 12.0);

    color(&mut cfg, "component.text_field.bg", "#1a1c20");
    color(&mut cfg, "component.text_field.border", "#4b5563");
    color(&mut cfg, "component.text_field.border_focus", "#6ea8e0");
    color(&mut cfg, "component.text_field.fg", "#e6e8eb");
    color(&mut cfg, "component.text_field.selection", "#315b8b");

    color(&mut cfg, "card", "#202328");
    color(&mut cfg, "muted", "#2a2d33");
    color(&mut cfg, "border", "#454d59");
    color(&mut cfg, "foreground", "#e6e8eb");
    color(&mut cfg, "muted-foreground", "#acb4bf");
    color(&mut cfg, "accent", "#4c88c7");
    color(&mut cfg, "ring", "#6ea8e0");

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

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{Color, Px};
    use fret_ui::Theme;

    use super::{EditorThemePresetV1, apply_editor_theme_preset_v1};
    use crate::primitives::EditorTokenKeys;

    #[test]
    fn default_preset_keeps_existing_editor_patch_baseline() {
        let mut app = App::new();
        apply_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

        let theme = Theme::global(&app);
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT),
            Some(Px(24.0))
        );
        assert_eq!(
            theme.metric_by_key("component.text_field.min_height"),
            Some(Px(24.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::SLIDER_THUMB_DIAMETER),
            Some(Px(12.0))
        );
    }

    #[test]
    fn imgui_like_dense_preset_overrides_density_and_field_chrome() {
        let mut app = App::new();
        apply_editor_theme_preset_v1(&mut app, EditorThemePresetV1::ImguiLikeDense);

        let theme = Theme::global(&app);
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT),
            Some(Px(22.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
            Some(Px(2.0))
        );
        assert_eq!(
            theme.metric_by_key("component.text_field.radius"),
            Some(Px(2.0))
        );
        assert_eq!(
            theme.color_by_key("component.text_field.bg"),
            Some(Color::from_srgb_hex_rgb(0x1a_1c_20))
        );
        assert_eq!(
            theme.color_by_key("border"),
            Some(Color::from_srgb_hex_rgb(0x45_4d_59))
        );
    }
}
