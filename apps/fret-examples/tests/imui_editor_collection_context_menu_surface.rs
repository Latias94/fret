#[test]
fn imui_editor_proof_demo_keeps_collection_context_menu_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "fn proof_collection_context_menu_line() -> String {",
        "fn proof_collection_context_menu_selection(",
        "fn authoring_parity_collection_context_menu_anchor_model<H: UiHost>(",
        "imui_editor_proof_demo.model.authoring_parity.collection_context_menu_anchor",
        "\"Right-click an asset or the collection background to open app-local collection actions.\"",
        "trigger.context_menu_requested()",
        "ui.open_popup_at(",
        "\"imui-editor-proof.authoring.imui.collection.context-menu\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.delete-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.dismiss\"",
        "Dismiss quick actions",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection context-menu proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_context_menu",
        "pub fn collection_context_menu",
        "struct ImUiCollectionContextMenu",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the context-menu slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
