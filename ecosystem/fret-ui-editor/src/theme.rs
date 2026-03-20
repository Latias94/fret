//! Editor-oriented theme patch helpers.
//!
//! These helpers are intentionally opt-in. They should be used by demos/apps that want an
//! editor-like density baseline without depending on a full design-system crate.

use std::any::TypeId;

use fret_core::WindowMetricsService;
use fret_ui::{Theme, ThemeConfig, UiHost};

use crate::primitives::EditorTokenKeys;

/// Installed editor preset configuration stored in app globals so apps can reapply editor-owned
/// token patches after a host-level theme reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorThemeInstallConfigV1 {
    pub preset: EditorThemePresetV1,
}

impl Default for EditorThemeInstallConfigV1 {
    fn default() -> Self {
        Self {
            preset: EditorThemePresetV1::Default,
        }
    }
}

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

/// Install an editor-oriented preset and remember it for later reapplication.
///
/// This is the preferred app-facing entry point when the host may reapply a base theme in response
/// to environment changes (for example, shadcn auto-sync on `WindowMetricsService` updates).
pub fn install_editor_theme_preset_v1<H: UiHost>(app: &mut H, preset: EditorThemePresetV1) {
    apply_editor_theme_preset_v1(app, preset);
    app.with_global_mut_untracked(EditorThemeInstallConfigV1::default, |stored, _app| {
        stored.preset = preset;
    });
}

/// Reapply the last installed editor preset after a host-level theme reset.
///
/// Returns the preset that was replayed, or `None` if no installed preset config exists.
pub fn reapply_installed_editor_theme_preset_v1<H: UiHost>(
    app: &mut H,
) -> Option<EditorThemePresetV1> {
    let preset = app.global::<EditorThemeInstallConfigV1>().copied()?.preset;
    apply_editor_theme_preset_v1(app, preset);
    Some(preset)
}

/// Reapply the installed editor preset when a `WindowMetricsService` change may have caused the
/// host app to rebuild its base theme.
///
/// This is the common "host changed first, editor patch second" ordering used by apps that keep a
/// host-owned theme in sync with environment light/dark preferences. If the host sync turns out to
/// be a no-op, the installed editor preset is not replayed again.
pub fn sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change<
    H: UiHost,
>(
    app: &mut H,
    changed: &[TypeId],
    sync_host_theme: impl FnOnce(&mut H),
) -> Option<EditorThemePresetV1> {
    if !changed.contains(&TypeId::of::<WindowMetricsService>()) {
        return None;
    }

    let theme_revision_before = Theme::global(&*app).revision();
    sync_host_theme(app);
    if Theme::global(&*app).revision() == theme_revision_before {
        return None;
    }
    reapply_installed_editor_theme_preset_v1(app)
}

/// Reapply the installed editor preset when `WindowMetricsService` changes and no host theme sync
/// callback is needed.
pub fn reapply_installed_editor_theme_preset_on_window_metrics_change<H: UiHost>(
    app: &mut H,
    changed: &[TypeId],
) -> Option<EditorThemePresetV1> {
    sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
        app,
        changed,
        |_app| {},
    )
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
    color(&mut cfg, "muted", "#171d26");
    color(&mut cfg, "border", "#2f3a48");
    color(&mut cfg, "foreground", "#edf3fa");
    color(&mut cfg, "muted-foreground", "#9eabbc");
    color(&mut cfg, "accent", "#355a86");
    color(&mut cfg, "ring", "#7faee8");
    color(&mut cfg, EditorTokenKeys::POPUP_BG, "#131b25");
    color(&mut cfg, EditorTokenKeys::POPUP_BORDER, "#46596c");
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
    metric(&mut cfg, EditorTokenKeys::VEC_AUTO_STACK_BELOW, 400.0);
    metric(&mut cfg, EditorTokenKeys::VEC_AXIS_MIN_WIDTH, 132.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_TRACK_HEIGHT, 3.0);
    metric(&mut cfg, EditorTokenKeys::SLIDER_THUMB_DIAMETER, 10.0);

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
    color(&mut cfg, "muted", "#2a2d33");
    color(&mut cfg, "border", "#454d59");
    color(&mut cfg, "foreground", "#e6e8eb");
    color(&mut cfg, "muted-foreground", "#acb4bf");
    color(&mut cfg, "accent", "#4c88c7");
    color(&mut cfg, "ring", "#6ea8e0");
    color(&mut cfg, EditorTokenKeys::POPUP_BG, "#24292f");
    color(&mut cfg, EditorTokenKeys::POPUP_BORDER, "#687686");
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

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{AppWindowId, Color, Px};
    use fret_ui::Theme;
    use fret_ui_shadcn::facade::themes::{
        ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york,
    };
    use std::any::TypeId;

    use super::{
        EditorThemePresetV1, apply_editor_theme_preset_v1, install_editor_theme_preset_v1,
        reapply_installed_editor_theme_preset_on_window_metrics_change,
        reapply_installed_editor_theme_preset_v1,
        sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change,
    };
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
            theme.metric_by_key(EditorTokenKeys::TEXT_FIELD_MIN_HEIGHT),
            Some(Px(24.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_LABEL_WIDTH),
            Some(Px(124.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_TRAILING_GAP),
            Some(Px(6.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_VALUE_MAX_WIDTH),
            Some(Px(1024.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_STATUS_SLOT_WIDTH),
            Some(Px(56.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_RESET_SLOT_WIDTH),
            Some(Px(24.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_GROUP_CONTENT_GAP),
            Some(Px(10.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_PANEL_GAP),
            Some(Px(14.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_GAP),
            Some(Px(12.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT),
            Some(Px(28.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::SLIDER_THUMB_DIAMETER),
            Some(Px(12.0))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            Some(Color::from_srgb_hex_rgb(0x14_1b_24))
        );
        assert_eq!(theme.metric_by_key("component.text_field.min_height"), None);
        assert_eq!(theme.color_by_key("component.text_field.bg"), None);
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG),
            Some(Color::from_srgb_hex_rgb(0x0f_15_1d))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::POPUP_BG),
            Some(Color::from_srgb_hex_rgb(0x13_1b_25))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::POPUP_BORDER),
            Some(Color::from_srgb_hex_rgb(0x46_59_6c))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_RADIUS),
            Some(Px(8.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_SHADOW_OFFSET_Y),
            Some(Px(6.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_SHADOW_BLUR),
            Some(Px(16.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_SHADOW_SPREAD),
            Some(Px(-4.0))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_BORDER),
            Some(Color::from_srgb_hex_rgb(0x3d_4d_5f))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BG),
            Some(Color::from_srgb_hex_rgb(0x24_34_45))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER),
            Some(Color::from_srgb_hex_rgb(0x5a_70_87))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_GROUP_BORDER),
            Some(Color::from_srgb_hex_rgb(0x33_41_4f))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG),
            Some(Color::from_srgb_hex_rgb(0x19_23_2e))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER),
            Some(Color::from_srgb_hex_rgb(0x38_48_57))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CONTROL_INVALID_BORDER),
            Some(Color::from_srgb_hex_rgb(0xc7_6f_77))
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
            theme.metric_by_key(EditorTokenKeys::PROPERTY_TRAILING_GAP),
            Some(Px(3.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_VALUE_MAX_WIDTH),
            Some(Px(840.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_STATUS_SLOT_WIDTH),
            Some(Px(48.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_RESET_SLOT_WIDTH),
            Some(Px(22.0))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::POPUP_BG),
            Some(Color::from_srgb_hex_rgb(0x24_29_2f))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::POPUP_BORDER),
            Some(Color::from_srgb_hex_rgb(0x68_76_86))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_RADIUS),
            Some(Px(4.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_SHADOW_OFFSET_Y),
            Some(Px(4.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_SHADOW_BLUR),
            Some(Px(12.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::POPUP_SHADOW_SPREAD),
            Some(Px(-3.0))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BG),
            Some(Color::from_srgb_hex_rgb(0x36_41_4c))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER),
            Some(Color::from_srgb_hex_rgb(0x72_82_94))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG),
            Some(Color::from_srgb_hex_rgb(0x28_30_39))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER),
            Some(Color::from_srgb_hex_rgb(0x56_62_6f))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_GROUP_CONTENT_GAP),
            Some(Px(6.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_PANEL_GAP),
            Some(Px(10.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_GAP),
            Some(Px(8.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT),
            Some(Px(24.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::TEXT_FIELD_RADIUS),
            Some(Px(2.0))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            Some(Color::from_srgb_hex_rgb(0x1a_1c_20))
        );
        assert_eq!(theme.metric_by_key("component.text_field.radius"), None);
        assert_eq!(theme.color_by_key("component.text_field.bg"), None);
        assert_eq!(
            theme.color_by_key("border"),
            Some(Color::from_srgb_hex_rgb(0x45_4d_59))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG),
            Some(Color::from_srgb_hex_rgb(0x1d_21_27))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BG),
            Some(Color::from_srgb_hex_rgb(0x36_41_4c))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_BORDER),
            Some(Color::from_srgb_hex_rgb(0x72_82_94))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_GROUP_BORDER),
            Some(Color::from_srgb_hex_rgb(0x47_51_5d))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG),
            Some(Color::from_srgb_hex_rgb(0x28_30_39))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER),
            Some(Color::from_srgb_hex_rgb(0x56_62_6f))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CONTROL_INVALID_BG),
            Some(Color::from_srgb_hex_rgb(0x36_22_25))
        );
    }

    #[test]
    fn installed_preset_can_be_reapplied_after_base_theme_reset() {
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
        install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

        let expected_field_bg = Some(Color::from_srgb_hex_rgb(0x14_1b_24));
        let expected_panel_bg = Some(Color::from_srgb_hex_rgb(0x0f_15_1d));
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            expected_field_bg
        );
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG),
            expected_panel_bg
        );

        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Light);
        assert_ne!(
            Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            expected_field_bg
        );

        assert_eq!(
            reapply_installed_editor_theme_preset_v1(&mut app),
            Some(EditorThemePresetV1::Default)
        );
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            expected_field_bg
        );
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG),
            expected_panel_bg
        );
    }

    #[test]
    fn window_metrics_helper_reapplies_after_host_theme_sync() {
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
        install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

        let expected_field_bg = Some(Color::from_srgb_hex_rgb(0x14_1b_24));
        let changed = [TypeId::of::<fret_core::WindowMetricsService>()];

        let replayed =
            sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
                &mut app,
                &changed,
                |app| {
                    apply_shadcn_new_york(app, ShadcnBaseColor::Slate, ShadcnColorScheme::Light);
                },
            );

        assert_eq!(replayed, Some(EditorThemePresetV1::Default));
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            expected_field_bg
        );
    }

    #[test]
    fn window_metrics_helper_skips_replay_when_host_theme_sync_is_noop() {
        let mut app = App::new();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
        app.with_global_mut(fret_core::WindowMetricsService::default, |svc, _app| {
            svc.set_color_scheme(window, Some(fret_core::ColorScheme::Dark));
        });
        let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
            &mut app,
            window,
            ShadcnBaseColor::Slate,
            ShadcnColorScheme::Dark,
        );
        install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

        let expected_field_bg = Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG);
        let before_rev = Theme::global(&app).revision();
        let changed = [TypeId::of::<fret_core::WindowMetricsService>()];

        let replayed =
            sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
                &mut app,
                &changed,
                |app| {
                    let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
                        app,
                        window,
                        ShadcnBaseColor::Slate,
                        ShadcnColorScheme::Dark,
                    );
                },
            );

        assert_eq!(replayed, None);
        assert_eq!(Theme::global(&app).revision(), before_rev);
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            expected_field_bg
        );
    }

    #[test]
    fn window_metrics_helper_ignores_unrelated_global_changes() {
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
        install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

        let expected_field_bg = Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG);
        let changed = [TypeId::of::<Theme>()];

        let replayed =
            reapply_installed_editor_theme_preset_on_window_metrics_change(&mut app, &changed);

        assert_eq!(replayed, None);
        assert_eq!(
            Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            expected_field_bg
        );
    }
}
