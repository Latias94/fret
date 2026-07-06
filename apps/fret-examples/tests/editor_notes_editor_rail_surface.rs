fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

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
        "fn editor_notes_section_text<",
        "fn editor_notes_paragraph_text<",
        "const TEST_ID_COLLECTION: &str = \"editor-notes-demo.collection\";",
        "const TEST_ID_COLLECTION_SUMMARY: &str = \"editor-notes-demo.collection.summary\";",
        "const TEST_ID_COLLECTION_LIST: &str = \"editor-notes-demo.collection.list\";",
        "fn editor_collection_row_label(",
        "fn editor_collection_status_label(",
        "shadcn::CardTitle::new(\"Scene collection\")",
        "Shell-mounted collection proof: choose an editor-owned surface",
        "editor_collection_row_label(",
        "editor_notes_readout_text(cx, editor_collection_status_label(selected))",
        ".test_id(TEST_ID_COLLECTION)",
        "editor_notes_paragraph_text(cx, ownership_note)",
        "editor_notes_section_text(cx, \"Active asset\")",
        "editor_notes_paragraph_text(cx, name_value.clone())",
        "editor_notes_section_text(cx, \"Inspector state\")",
        "editor_notes_readout_text(cx, note_summary.clone())",
        "editor_notes_readout_text(cx, format!(\"Last action: {outcome_label}\"))",
        "editor_notes_section_text(cx, \"Committed notes\")",
        "editor_notes_paragraph_text(cx, committed_notes_intro)",
        "editor_notes_paragraph_text(cx, preview_text)",
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
        "editor_notes_readout_text(\n                                                cx,\n                                                draft_status_label.clone(),\n                                            )",
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
        "ui::text(editor_collection_status_label(selected))",
        "ui::text(ownership_note)",
        "ui::text(\"Active asset\")",
        "ui::text(name_value.clone())",
        "ui::text(\"Inspector state\")",
        "ui::text(note_summary.clone())",
        "ui::text(format!(\"Last action: {outcome_label}\"))",
        "ui::text(\"Committed notes\")",
        "ui::text(committed_notes_intro)",
        "ui::text(preview_text)",
    ] {
        assert!(
            !source.contains(needle),
            "editor notes demo should keep inspector row text on semantic roles; unexpected `{needle}`"
        );
    }
}

#[test]
fn editor_notes_demo_model_writes_stay_behind_owner_helpers() {
    let source = include_str!("../src/editor_notes_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("editor notes demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::ModelStore;",
        "structEditorNotesModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fnset_text(&mutself,model:&Model<String>,value:implInto<String>)->bool{",
        "letmutowner=EditorNotesModelOwner::new(host.models_mut());",
        "owner.set_text(&notes_outcome_model,next",
        "owner.set_text(&notes_outcome_model,\"Committed\"",
        "owner.set_text(&notes_outcome_model,\"Canceled\"",
        "owner.set_text(&summary_status_model,draft_commit_status.clone()",
        "owner.set_text(&summary_status_model,draft_discard_status.clone()",
        "owner.set_text(&summary_status_model,summary_status_next.clone()",
    ] {
        assert!(
            compact_source.contains(needle),
            "editor notes demo should keep shared-model writes behind a named owner helper; missing `{needle}`"
        );
    }

    for forbidden in [
        "models_mut().update(",
        "models_mut().update::<",
        "models_mut().update_any(",
        "models_mut().update_any::<",
        "ModelStore::update(",
        "ModelStore::update::<",
        "ModelStore::update_any(",
        "ModelStore::update_any::<",
        "<ModelStore>::update(",
        "<ModelStore>::update::<",
        "<ModelStore>::update_any(",
        "<ModelStore>::update_any::<",
        "fneditor_notes_host_update_model",
        "fneditor_notes_host_set_model",
        "fneditor_notes_host_set_text",
    ] {
        assert!(
            !compact_production.contains(forbidden),
            "editor notes production code should not bypass the owner helper with `{forbidden}`"
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
