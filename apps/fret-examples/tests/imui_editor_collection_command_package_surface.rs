#[test]
fn imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "struct ProofCollectionDuplicateResult {",
        "fn proof_collection_command_package_line() -> String {",
        "fn proof_collection_command_status_line(status: &str) -> String {",
        "fn proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_duplicate_selection(",
        "fn proof_collection_duplicate_status(",
        "fn proof_collection_begin_inline_rename_in_app(",
        "fn authoring_parity_collection_command_status_model<H: UiHost>(",
        "imui_editor_proof_demo.model.authoring_parity.collection_command_status",
        "\"Duplicate, delete, rename, and select-all stay inside one app-owned collection command package; duplicate/delete/rename now route across keyboard, explicit buttons, and context menu without widening shared IMUI helpers.\"",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected\"",
        "\"Command status: {status}\"",
        "proof_collection_duplicate_shortcut_matches(",
        "KeyCode::KeyD",
        "shortcut: Some(Arc::from(\"Primary+D\"))",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection command-package slice explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_command_package",
        "pub fn collection_command_package",
        "pub fn collection_duplicate_selected",
        "struct ImUiCollectionCommandPackage",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the command-package slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
