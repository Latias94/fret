#[test]
fn imui_editor_proof_demo_keeps_collection_select_all_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "fn proof_collection_select_all_line() -> String {",
        "fn proof_collection_select_all_shortcut_matches(",
        "fn proof_collection_select_all_selection(",
        "\"Primary+A selects all visible assets inside the focused collection scope.\"",
        "proof_collection_select_all_shortcut_matches(",
        "KeyCode::KeyA",
        "proof_collection_select_all_selection(",
        "ui.text(proof_collection_select_all_line());",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection select-all proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_select_all",
        "pub fn collection_select_all",
        "struct ImUiCollectionSelectAll",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the collection select-all slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
