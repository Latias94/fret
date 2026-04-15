#[test]
fn workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots() {
    let source = include_str!("../src/workspace_shell_demo.rs");

    for needle in [
        "fn workspace_shell_editor_rail<'a, Cx>(",
        "Cx: fret::app::ElementContextAccess<'a, App>,",
        "let right = cx.keyed(\"workspace_shell.right\"",
        "workspace_shell_editor_rail(",
        "InspectorPanel::new(None)",
        ".into_element_in(",
        "PropertyGroup::new(\"Selection\")",
        "PropertyGroup::new(\"Shell\")",
        "PropertyGrid::new().into_element_in(",
        ".right(right)",
        "\"workspace-shell-editor-rail\"",
    ] {
        assert!(
            source.contains(needle),
            "workspace shell demo should keep the editor-rail composition explicit; missing `{needle}`"
        );
    }
}
