//! Editor-oriented theme patch helpers.
//!
//! These helpers are intentionally opt-in. They should be used by demos/apps that want an
//! editor-like density baseline without depending on a full design-system crate.

mod install;
mod patches;
mod presets;
mod sync;

pub use install::{
    EditorThemeInstallConfig, apply_editor_theme_preset, install_editor_theme_preset,
    installed_editor_theme_preset, reapply_installed_editor_theme_preset,
};
pub use presets::{EDITOR_THEME_PRESETS, EditorThemePreset};
pub use sync::reapply_installed_editor_theme_preset_on_window_metrics_change;
pub use sync::sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change;

use patches::{editor_theme_patch, editor_theme_preset_overrides};

#[cfg(test)]
mod tests;
