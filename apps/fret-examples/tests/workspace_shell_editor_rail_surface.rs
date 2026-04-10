#[test]
fn workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots() {
    let source = include_str!("../src/workspace_shell_demo.rs");

    for needle in [
        "let right = cx.keyed(\"workspace_shell.right\"",
        "InspectorPanel::new(None)",
        "PropertyGroup::new(\"Selection\")",
        "PropertyGroup::new(\"Shell\")",
        ".right(right)",
        "\"workspace-shell-editor-rail\"",
    ] {
        assert!(
            source.contains(needle),
            "workspace shell demo should keep the editor-rail composition explicit; missing `{needle}`"
        );
    }
}
