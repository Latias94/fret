#[test]
fn imui_editor_proof_demo_keeps_collection_inline_rename_app_owned_and_explicit() {
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename.rs"),
    );

    for needle in [
        "fn proof_collection_rename_line() -> String {",
        "fn proof_collection_rename_shortcut_matches(",
        "fn proof_collection_begin_rename_session(",
        "fn proof_collection_begin_inline_rename_in_app(",
        "fn proof_collection_commit_rename(",
        "fn proof_collection_inline_rename_focus_state<",
        "fn proof_collection_sync_inline_rename_focus<",
        "fn proof_collection_restore_focus_after_inline_rename(",
        "\"F2, the explicit rename button, or the context menu starts an app-local inline rename editor for the current active asset.\"",
        "proof_collection_rename_shortcut_matches(",
        "KeyCode::F2",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.rename\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline\"",
        "\"Rename active asset\"",
        "TextField::new(",
        "TextFieldOptions {",
        "EditorTextSelectionBehavior::SelectAllOnFocus",
        "TextFieldBlurBehavior::Cancel",
        "proof_collection_inline_rename_focus_state(",
        "proof_collection_sync_inline_rename_focus(",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_rename_line(),",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection inline-rename proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "ui.begin_popup_modal_with_options(",
        "PROOF_COLLECTION_RENAME_COMMIT_COMMAND",
        "PROOF_COLLECTION_RENAME_CANCEL_COMMAND",
        "\"imui-editor-proof.authoring.imui.collection.rename.input\"",
        "\"imui-editor-proof.authoring.imui.collection.rename.commit\"",
        "\"imui-editor-proof.authoring.imui.collection.rename.cancel\"",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not keep the superseded modal rename surface around; unexpected `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_rename",
        "pub fn collection_rename",
        "struct ImUiCollectionRename",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the collection inline-rename slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
