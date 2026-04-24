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
        "const TEST_ID_COLLECTION: &str = \"editor-notes-demo.collection\";",
        "const TEST_ID_COLLECTION_SUMMARY: &str = \"editor-notes-demo.collection.summary\";",
        "const TEST_ID_COLLECTION_LIST: &str = \"editor-notes-demo.collection.list\";",
        "fn editor_collection_row_label(",
        "fn editor_collection_status_label(",
        "shadcn::CardTitle::new(\"Scene collection\")",
        "Shell-mounted collection proof: choose an editor-owned surface",
        "editor_collection_row_label(",
        "ui::text(editor_collection_status_label(selected))",
        ".test_id(TEST_ID_COLLECTION)",
        "const TEST_ID_NOTES_DRAFT_STATUS: &str = \"editor-notes-demo.inspector.notes.draft-status\";",
        "const TEST_ID_SUMMARY_COMMAND: &str = \"editor-notes-demo.inspector.summary-command\";",
        "const TEST_ID_SUMMARY_STATUS: &str = \"editor-notes-demo.inspector.summary-status\";",
        "fn editor_asset_summary_command_status(",
        "fn editor_notes_draft_status_label(",
        "summary_status_model: Model<String>",
        "shadcn::Button::new(\"Copy asset summary\")",
        ".test_id(TEST_ID_SUMMARY_COMMAND)",
        ".test_id(TEST_ID_SUMMARY_STATUS)",
        "cx.text(\"Draft status\")",
        ".test_id(TEST_ID_NOTES_DRAFT_STATUS)",
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
