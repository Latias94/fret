//! Editor-oriented theme patch helpers.
//!
//! These helpers are intentionally opt-in. They should be used by demos/apps that want an
//! editor-like density baseline without depending on a full design-system crate.

mod install;
mod patches;
mod presets;
mod sync;

pub use install::{
    EditorThemeInstallConfigV1, apply_editor_theme_preset_v1, install_editor_theme_preset_v1,
    installed_editor_theme_preset_v1, reapply_installed_editor_theme_preset_v1,
};
pub use presets::{EDITOR_THEME_PRESETS_V1, EditorThemePresetV1};
pub use sync::reapply_installed_editor_theme_preset_on_window_metrics_change;
pub use sync::sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change;

use patches::{editor_theme_patch_v1, editor_theme_preset_overrides_v1};

#[cfg(test)]
mod tests;
