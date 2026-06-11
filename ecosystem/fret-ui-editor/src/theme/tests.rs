use fret_app::App;
use fret_core::{AppWindowId, Color, Px};
use fret_ui::Theme;
use fret_ui_shadcn::facade::themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};
use std::any::TypeId;

use super::{
    EDITOR_THEME_PRESETS_V1, EditorThemePresetV1, apply_editor_theme_preset_v1,
    install_editor_theme_preset_v1, installed_editor_theme_preset_v1,
    reapply_installed_editor_theme_preset_on_window_metrics_change,
    reapply_installed_editor_theme_preset_v1,
    sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change,
};
use crate::primitives::EditorTokenKeys;

mod metadata;
mod preset_baseline;
mod replay;
