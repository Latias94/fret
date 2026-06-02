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

    pub const fn picker_status_label(self) -> &'static str {
        match self {
            Self::Default => "24px",
            Self::ImguiLikeDense => "22px",
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
mod tests;
