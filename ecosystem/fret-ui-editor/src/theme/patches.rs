use fret_ui::ThemeConfig;

use super::EditorThemePresetV1;

mod default;
mod dense;
mod helpers;

pub(super) use default::editor_theme_patch_v1;
use dense::imgui_like_dense_patch_v1;
pub(super) use helpers::{color, metric};

pub(super) fn editor_theme_preset_overrides_v1(preset: EditorThemePresetV1) -> Option<ThemeConfig> {
    match preset {
        EditorThemePresetV1::Default => None,
        EditorThemePresetV1::ImguiLikeDense => Some(imgui_like_dense_patch_v1()),
    }
}
