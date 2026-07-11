use fret_ui::ThemeConfig;

use super::EditorThemePreset;

mod default;
mod dense;
mod helpers;

pub(super) use default::editor_theme_patch;
use dense::imgui_like_dense_patch;
pub(super) use helpers::{color, metric};

pub(super) fn editor_theme_preset_overrides(preset: EditorThemePreset) -> Option<ThemeConfig> {
    match preset {
        EditorThemePreset::Default => None,
        EditorThemePreset::ImguiLikeDense => Some(imgui_like_dense_patch()),
    }
}
