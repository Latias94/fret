#[test]
fn imui_editor_proof_demo_keeps_collection_keyboard_owner_app_owned_and_explicit() {
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let keyboard_source = include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs");
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select.rs"),
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
        "struct ProofCollectionKeyboardState {",
        "fn proof_collection_active_line(",
        "fn proof_collection_keyboard_selection(",
        "fn proof_collection_keyboard_next_index(",
        "fn proof_collection_keyboard_move_selection(",
        "imui_editor_proof_demo.model.authoring_parity.collection_keyboard",
        "pub(super) struct ProofCollectionKeyboardHandlerModels",
        "pub(super) fn install_collection_keyboard_handler(",
        "install_collection_keyboard_handler(",
        "cx.key_on_key_down_for(",
        "down.ime_composing",
        "proof_collection_delete_key_matches(down.key)",
        "proof_collection_rename_shortcut_matches(down.key, down.modifiers)",
        "proof_collection_select_all_shortcut_matches(down.key, down.modifiers)",
        "proof_collection_duplicate_shortcut_matches(down.key, down.modifiers)",
        "host.notify(acx);",
        "host.request_focus(acx.target);",
        "state.active_id = next_selection.first_selected().cloned();",
        "state.active_id = None;",
        "\"Active tile: none. Click background to focus the collection scope, then use Arrow/Home/End to drive selection app-locally.\"",
        "\"Active tile: {}. Shift+Arrow/Home/End extends from the current anchor; Escape clears the selection without widening shared IMUI helper ownership.\"",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection keyboard-owner proof explicit and app-owned; missing `{needle}`"
        );
    }

    assert!(
        !collection_source.contains("cx.key_on_key_down_for("),
        "collection root should delegate keyboard handler installation to the keyboard owner"
    );
    assert!(
        keyboard_source.contains("cx.key_on_key_down_for("),
        "collection keyboard owner should keep the scope key handler installation explicit"
    );

    for needle in [
        "fret_ui_kit::imui::collection_keyboard_owner",
        "pub fn collection_keyboard_owner",
        "pub fn set_next_collection_shortcut",
        "SetNextItemShortcut",
        "SetItemKeyOwner",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the keyboard-owner slice is a shared helper or generic key-owner facade; unexpected `{needle}`"
        );
    }
}
