#[test]
fn imui_editor_proof_demo_keeps_collection_select_all_app_owned_and_explicit() {
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
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
        "fn proof_collection_select_all_line() -> String {",
        "fn proof_collection_select_all_shortcut_matches(",
        "fn proof_collection_select_all_selection(",
        "\"Primary+A selects all visible assets inside the focused collection scope.\"",
        "proof_collection_select_all_shortcut_matches(",
        "KeyCode::KeyA",
        "proof_collection_select_all_selection(",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_select_all_line(),",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
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
