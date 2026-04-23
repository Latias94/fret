#[test]
fn imui_editor_proof_demo_keeps_collection_delete_action_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "struct ProofCollectionDeleteResult {",
        "fn proof_collection_assets_line(",
        "fn proof_collection_delete_key_matches(",
        "fn proof_collection_delete_selection(",
        "fn authoring_parity_collection_assets_model<H: UiHost>(",
        "imui_editor_proof_demo.model.authoring_parity.collection_assets",
        "\"Delete selected assets\"",
        "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
        "\"Assets: {}. Press Delete/Backspace or use the explicit action button to remove the selected set app-locally.\"",
        "proof_collection_delete_key_matches(down.key)",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection delete-action proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_delete_action",
        "pub fn collection_delete_action",
        "pub fn delete_selected_assets",
        "struct ImUiCollectionDeleteAction",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the delete-action slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
