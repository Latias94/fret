use fret_ui::{Theme, UiHost};

use super::presets::EditorThemePresetV1;
use super::{editor_theme_patch_v1, editor_theme_preset_overrides_v1};

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
/// to environment changes.
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
