#[test]
fn editor_notes_demo_composes_shell_mounted_rails_through_workspace_frame_slots() {
    let source = include_str!("../src/editor_notes_demo.rs");

    for needle in [
        "let theme = cx.theme_snapshot();",
        "use fret_workspace::WorkspaceFrame;",
        "let left_rail = ui::container(|_cx| [selection_panel])",
        "let right_rail = ui::container(|_cx| [inspector])",
        "WorkspaceFrame::new(center)",
        ".left(left_rail)",
        ".right(right_rail)",
        "render_center_panel(",
        "InspectorPanel::new(None)",
        "PropertyGroup::new(\"Metadata\")",
        "\"editor-notes-demo.right-rail\"",
    ] {
        assert!(
            source.contains(needle),
            "editor notes demo should keep the shell-mounted editor-rail composition explicit; missing `{needle}`"
        );
    }

    assert!(
        !source.contains("Theme::global(&*cx.app).snapshot()"),
        "editor notes demo should use the app-facing theme snapshot helper instead of reading theme through cx.app",
    );
}
