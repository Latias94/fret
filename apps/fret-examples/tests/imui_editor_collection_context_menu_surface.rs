#[test]
fn imui_editor_proof_demo_keeps_collection_context_menu_app_owned_and_explicit() {
    let context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs");
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/models.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/readouts.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename.rs"),
    );

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
        "let duplicate_from_menu = ui.menu_item_with_options(",
        "proof_collection_duplicate_selection(",
        "proof_collection_duplicate_status(&duplicate.duplicated_assets)",
        ".update(&models.assets, |state| {",
        ".update(&models.selection, |state| {",
        ".update(&models.keyboard, |state| {",
        ".update(&models.command_status, |status| {",
        "let rename_from_menu = ui.menu_item_with_options(",
        "proof_collection_begin_inline_rename_in_app(",
        "&models.rename_session,",
        "&models.rename_draft,",
        "&models.rename_focus_pending,",
        "&models.rename_status,",
        "let delete_from_menu = ui.menu_item_with_options(",
        "proof_collection_delete_selection(",
        "proof_collection_delete_status(&delete.deleted_assets)",
        "\"Dismiss quick actions\"",
    ] {
        assert!(
            context_menu_source.contains(needle),
            "collection context-menu owner should route menu actions through app-owned state transitions; missing `{needle}`"
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
