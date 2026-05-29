//! Editor-oriented theme patch helpers.
//!
//! These helpers are intentionally opt-in. They should be used by demos/apps that want an
//! editor-like density baseline without depending on a full design-system crate.

use std::any::TypeId;

use fret_core::WindowMetricsService;
use fret_ui::{Theme, UiHost};

use patches::{editor_theme_patch_v1, editor_theme_preset_overrides_v1};

mod patches;

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

/// Stable editor theme preset order for editor tools and diagnostics.
pub const EDITOR_THEME_PRESETS_V1: [EditorThemePresetV1; 2] = [
    EditorThemePresetV1::Default,
    EditorThemePresetV1::ImguiLikeDense,
];

impl EditorThemePresetV1 {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::ImguiLikeDense => "imgui_like_dense",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::ImguiLikeDense => "ImGui-like dense",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let normalized = key.trim().to_ascii_lowercase().replace('-', "_");
        EDITOR_THEME_PRESETS_V1
            .iter()
            .copied()
            .find(|preset| preset.key() == normalized.as_str())
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

/// Returns the last installed editor theme preset, if app code opted into install/replay tracking.
pub fn installed_editor_theme_preset_v1<H: UiHost>(app: &H) -> Option<EditorThemePresetV1> {
    app.global::<EditorThemeInstallConfigV1>()
        .copied()
        .map(|stored| stored.preset)
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
        EDITOR_THEME_PRESETS_V1, EditorThemePresetV1, apply_editor_theme_preset_v1,
        install_editor_theme_preset_v1, installed_editor_theme_preset_v1,
        reapply_installed_editor_theme_preset_on_window_metrics_change,
        reapply_installed_editor_theme_preset_v1,
        sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change,
    };
    use crate::primitives::EditorTokenKeys;

    #[test]
    fn editor_theme_preset_metadata_is_stable_for_tools() {
        assert_eq!(
            EDITOR_THEME_PRESETS_V1,
            [
                EditorThemePresetV1::Default,
                EditorThemePresetV1::ImguiLikeDense
            ]
        );
        assert_eq!(EditorThemePresetV1::Default.key(), "default");
        assert_eq!(EditorThemePresetV1::Default.label(), "Default");
        assert_eq!(
            EditorThemePresetV1::ImguiLikeDense.key(),
            "imgui_like_dense"
        );
        assert_eq!(
            EditorThemePresetV1::ImguiLikeDense.label(),
            "ImGui-like dense"
        );
        assert_eq!(
            EditorThemePresetV1::from_key("imgui_like_dense"),
            Some(EditorThemePresetV1::ImguiLikeDense)
        );
        assert_eq!(
            EditorThemePresetV1::from_key("IMGUI-LIKE-DENSE"),
            Some(EditorThemePresetV1::ImguiLikeDense)
        );
        assert_eq!(EditorThemePresetV1::from_key("unknown"), None);
    }

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
            theme.color_by_key(EditorTokenKeys::SLIDER_TRACK_BG),
            Some(Color::from_srgb_hex_rgb(0x17_1d_26))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::SLIDER_FILL_BG),
            Some(Color::from_srgb_hex_rgb(0x35_5a_86))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::SLIDER_THUMB_BG),
            Some(Color::from_srgb_hex_rgb(0x14_1b_24))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::SLIDER_THUMB_BORDER),
            Some(Color::from_srgb_hex_rgb(0x3b_47_58))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_BG),
            Some(Color::from_srgb_hex_rgb(0x14_1b_24))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_CHECKED_BG),
            Some(Color::from_srgb_hex_rgb(0x35_5a_86))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_CHECKED_FG),
            Some(Color::from_srgb_hex_rgb(0xed_f3_fa))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_RING),
            Some(Color::from_srgb_hex_rgb(0x7f_ae_e8))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
            Some(Color::from_srgb_hex_rgb(0x14_1b_24))
        );
        assert_eq!(
            theme.color_by_key("background"),
            Some(Color::from_srgb_hex_rgb(0x0c_11_18))
        );
        assert_eq!(
            theme.color_by_key("input"),
            Some(Color::from_srgb_hex_rgb(0x3b_47_58))
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
            theme.color_by_key(EditorTokenKeys::POPUP_SHADOW_COLOR),
            Some(Color::from_srgb_hex_rgb(0x17_1d_26))
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
            theme.metric_by_key(EditorTokenKeys::PROPERTY_PANEL_RADIUS),
            Some(Px(6.0))
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
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED),
            Some(Px(0.02))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SLOW_MULTIPLIER),
            Some(Px(0.1))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_FAST_MULTIPLIER),
            Some(Px(10.0))
        );
        assert_eq!(
            theme.metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
            Some(Px(4.0))
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
            theme.color_by_key(EditorTokenKeys::POPUP_SHADOW_COLOR),
            Some(Color::from_srgb_hex_rgb(0x2a_2d_33))
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
            theme.metric_by_key(EditorTokenKeys::PROPERTY_PANEL_RADIUS),
            Some(Px(2.0))
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
            theme.color_by_key(EditorTokenKeys::SLIDER_TRACK_BG),
            Some(Color::from_srgb_hex_rgb(0x2a_2d_33))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::SLIDER_FILL_BG),
            Some(Color::from_srgb_hex_rgb(0x4c_88_c7))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::SLIDER_THUMB_BG),
            Some(Color::from_srgb_hex_rgb(0x1a_1c_20))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::SLIDER_THUMB_BORDER),
            Some(Color::from_srgb_hex_rgb(0x4b_55_63))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_BG),
            Some(Color::from_srgb_hex_rgb(0x1a_1c_20))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_CHECKED_BG),
            Some(Color::from_srgb_hex_rgb(0x4c_88_c7))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_CHECKED_FG),
            Some(Color::from_srgb_hex_rgb(0xe6_e8_eb))
        );
        assert_eq!(
            theme.color_by_key(EditorTokenKeys::CHECKBOX_RING),
            Some(Color::from_srgb_hex_rgb(0x6e_a8_e0))
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
            theme.color_by_key("background"),
            Some(Color::from_srgb_hex_rgb(0x17_1a_1f))
        );
        assert_eq!(
            theme.color_by_key("input"),
            Some(Color::from_srgb_hex_rgb(0x4b_55_63))
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
    fn default_preset_resets_dense_numeric_scrub_tokens() {
        let mut app = App::new();
        apply_editor_theme_preset_v1(&mut app, EditorThemePresetV1::ImguiLikeDense);
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED),
            Some(Px(0.035))
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
            Some(Px(2.0))
        );

        apply_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED),
            Some(Px(0.02))
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
            Some(Px(4.0))
        );
    }

    #[test]
    fn installed_preset_can_be_reapplied_after_base_theme_reset() {
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
        install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);
        assert_eq!(
            installed_editor_theme_preset_v1(&app),
            Some(EditorThemePresetV1::Default)
        );

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
