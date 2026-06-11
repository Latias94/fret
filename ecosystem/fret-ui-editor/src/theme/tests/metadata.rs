use super::*;

#[test]
fn editor_theme_preset_metadata_is_stable_for_tools() {
    assert_eq!(
        EDITOR_THEME_PRESETS_V1,
        [
            EditorThemePresetV1::Default,
            EditorThemePresetV1::ImguiLikeDense
        ]
    );
    assert_eq!(EditorThemePresetV1::Default.key(), "default");
    assert_eq!(EditorThemePresetV1::Default.label(), "Default");
    assert_eq!(
        EditorThemePresetV1::ImguiLikeDense.key(),
        "imgui_like_dense"
    );
    assert_eq!(
        EditorThemePresetV1::ImguiLikeDense.label(),
        "ImGui-like dense"
    );
    assert_eq!(EditorThemePresetV1::Default.picker_status_label(), "24px");
    assert_eq!(
        EditorThemePresetV1::ImguiLikeDense.picker_status_label(),
        "22px"
    );
    assert_eq!(
        EditorThemePresetV1::from_key("imgui_like_dense"),
        Some(EditorThemePresetV1::ImguiLikeDense)
    );
    assert_eq!(
        EditorThemePresetV1::from_key("IMGUI-LIKE-DENSE"),
        Some(EditorThemePresetV1::ImguiLikeDense)
    );
    assert_eq!(EditorThemePresetV1::from_key("unknown"), None);
}
