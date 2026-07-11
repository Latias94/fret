use fret_ui::{Theme, UiHost};

use super::presets::EditorThemePreset;
use super::{editor_theme_patch, editor_theme_preset_overrides};

/// Installed editor preset configuration stored in app globals so apps can reapply editor-owned
/// token patches after a host-level theme reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorThemeInstallConfig {
    pub preset: EditorThemePreset,
}

impl Default for EditorThemeInstallConfig {
    fn default() -> Self {
        Self {
            preset: EditorThemePreset::Default,
        }
    }
}

/// Apply an editor-oriented preset layered on top of the current theme.
///
/// This is designed as a patch on top of an existing theme (e.g. shadcn New York) and is safe to
/// call multiple times.
pub fn apply_editor_theme_preset<H: UiHost>(app: &mut H, preset: EditorThemePreset) {
    Theme::with_global_mut(app, |theme| {
        theme.apply_config_patch(&editor_theme_patch());

        if let Some(preset_cfg) = editor_theme_preset_overrides(preset) {
            theme.apply_config_patch(&preset_cfg);
        }
    });
}

/// Install an editor-oriented preset and remember it for later reapplication.
///
/// This is the preferred app-facing entry point when the host may reapply a base theme in response
/// to environment changes.
pub fn install_editor_theme_preset<H: UiHost>(app: &mut H, preset: EditorThemePreset) {
    apply_editor_theme_preset(app, preset);
    app.with_global_mut_untracked(EditorThemeInstallConfig::default, |stored, _app| {
        stored.preset = preset;
    });
}

/// Returns the last installed editor theme preset, if app code opted into install/replay tracking.
pub fn installed_editor_theme_preset<H: UiHost>(app: &H) -> Option<EditorThemePreset> {
    app.global::<EditorThemeInstallConfig>()
        .copied()
        .map(|stored| stored.preset)
}

/// Reapply the last installed editor preset after a host-level theme reset.
///
/// Returns the preset that was replayed, or `None` if no installed preset config exists.
pub fn reapply_installed_editor_theme_preset<H: UiHost>(app: &mut H) -> Option<EditorThemePreset> {
    let preset = app.global::<EditorThemeInstallConfig>().copied()?.preset;
    apply_editor_theme_preset(app, preset);
    Some(preset)
}
