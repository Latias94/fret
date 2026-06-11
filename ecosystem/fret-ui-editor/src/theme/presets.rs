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
