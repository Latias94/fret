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
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn editor_notes_readout_text<",
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
        "const TEST_ID_DRAFT_COMMIT_COMMAND: &str = \"editor-notes-demo.inspector.notes.commit-draft\";",
        "const TEST_ID_DRAFT_DISCARD_COMMAND: &str = \"editor-notes-demo.inspector.notes.discard-draft\";",
        "const TEST_ID_SUMMARY_COMMAND: &str = \"editor-notes-demo.inspector.summary-command\";",
        "const TEST_ID_SUMMARY_STATUS: &str = \"editor-notes-demo.inspector.summary-status\";",
        "fn editor_asset_summary_command_status(",
        "fn editor_notes_draft_status_label(",
        "fn editor_notes_draft_action_status(",
        "TextFieldDraftController::new",
        "draft_controller: Some(draft_controller.clone())",
        ".commit(host, action_cx)",
        ".discard(host, action_cx)",
        "summary_status_model: Model<String>",
        "shadcn::Button::new(\"Copy asset summary\")",
        ".test_id(TEST_ID_SUMMARY_COMMAND)",
        ".test_id(TEST_ID_SUMMARY_STATUS)",
        "row_cx.label_text(cx, \"Draft status\")",
        "editor_notes_readout_text(cx, draft_status_label.clone())",
        ".test_id(TEST_ID_NOTES_DRAFT_STATUS)",
        "row_cx.label_text(cx, \"Draft actions\")",
        "row_cx.label_text(cx, \"Summary command\")",
        "row_cx.label_text(cx, \"Summary status\")",
        "editor_notes_readout_text(cx, committed_label.clone())",
        "editor_notes_readout_text(cx, outcome_label.clone())",
        "editor_notes_readout_text(cx, summary_status.clone())",
        "shadcn::Button::new(\"Commit draft\")",
        "shadcn::Button::new(\"Discard draft\")",
        ".test_id(TEST_ID_DRAFT_COMMIT_COMMAND)",
        ".test_id(TEST_ID_DRAFT_DISCARD_COMMAND)",
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

    for needle in [
        "cx.text(\"Draft status\")",
        "cx.text(\"Draft actions\")",
        "cx.text(committed_label.clone())",
        "cx.text(outcome_label.clone())",
        "cx.text(draft_status_label.clone())",
        "cx.text(summary_status.clone())",
    ] {
        assert!(
            !source.contains(needle),
            "editor notes demo should keep inspector row text on semantic roles; unexpected `{needle}`"
        );
    }
}

#[test]
fn editor_notes_demo_draft_controller_diag_script_clicks_app_owned_commit_and_discard() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json"
    );

    for needle in [
        "\"schema_version\": 2",
        "\"editor-notes-demo-draft-controller-proof\"",
        "\"draft_controller\"",
        "\"editor-notes-demo.inspector.notes\"",
        "\"editor-notes-demo.inspector.notes.commit-draft\"",
        "\"editor-notes-demo.inspector.notes.discard-draft\"",
        "\"Controller commit A\\nController commit B\\nController commit C\"",
        "\"Controller discard pending\"",
        "\"Draft committed: Material\"",
        "\"Draft discarded: Material\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"capture_screenshot\"",
        "\"type\": \"capture_bundle\"",
    ] {
        assert!(
            script.contains(needle),
            "editor notes draft-controller diag script should keep the app-owned commit/discard proof reviewable; missing `{needle}`"
        );
    }
}

#[test]
fn editor_notes_demo_selection_sync_diag_script_keeps_product_workflow_reviewable() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-selection-sync.json"
    );
    let suite = include_str!("../../../tools/diag-scripts/suites/editor-notes-demo/suite.json");

    for needle in [
        "\"schema_version\": 2",
        "\"editor-notes-demo-selection-sync\"",
        "\"editor-notes-demo.collection.summary\"",
        "\"active: Key Light\"",
        "\"active: Camera\"",
        "\"active: Material\"",
        "\"editor-notes-demo.selection.light\"",
        "\"editor-notes-demo.selection.camera\"",
        "\"editor-notes-demo.selection.material\"",
        "\"editor-notes-demo.inspector.name\"",
        "\"Weathered Steel\"",
        "\"Key Light A\"",
        "\"ShotCam_Main\"",
        "\"editor-notes-demo.inspector.notes\"",
        "\"Ready to copy summary for Key Light.\"",
        "\"Copied summary: Key Light\"",
        "\"Ready to copy summary for Camera.\"",
        "\"Copied summary: Camera\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"capture_screenshot\"",
        "\"type\": \"capture_bundle\"",
    ] {
        assert!(
            script.contains(needle),
            "editor notes selection-sync diag script should keep the asset selection -> inspector workflow reviewable; missing `{needle}`"
        );
    }

    assert!(
        suite.contains(
            "tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-selection-sync.json"
        ),
        "editor-notes-demo suite should include the selection-sync product workflow script",
    );
}
