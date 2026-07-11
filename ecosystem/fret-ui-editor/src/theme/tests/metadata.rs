use super::*;

#[test]
fn editor_theme_preset_metadata_is_stable_for_tools() {
    assert_eq!(
        EDITOR_THEME_PRESETS,
        [
            EditorThemePreset::Default,
            EditorThemePreset::ImguiLikeDense
        ]
    );
    assert_eq!(EditorThemePreset::Default.key(), "default");
    assert_eq!(EditorThemePreset::Default.label(), "Default");
    assert_eq!(EditorThemePreset::ImguiLikeDense.key(), "imgui_like_dense");
    assert_eq!(
        EditorThemePreset::ImguiLikeDense.label(),
        "ImGui-like dense"
    );
    assert_eq!(EditorThemePreset::Default.picker_status_label(), "24px");
    assert_eq!(
        EditorThemePreset::ImguiLikeDense.picker_status_label(),
        "22px"
    );
    assert_eq!(
        EditorThemePreset::from_key("imgui_like_dense"),
        Some(EditorThemePreset::ImguiLikeDense)
    );
    assert_eq!(
        EditorThemePreset::from_key("IMGUI-LIKE-DENSE"),
        Some(EditorThemePreset::ImguiLikeDense)
    );
    assert_eq!(EditorThemePreset::from_key("unknown"), None);
}
