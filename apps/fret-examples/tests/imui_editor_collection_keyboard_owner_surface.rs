#[test]
fn imui_editor_proof_demo_keeps_collection_keyboard_owner_app_owned_and_explicit() {
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let keyboard_source = include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs");
    let keyboard_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/keyboard/actions.rs");
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/tile.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests/fixtures.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests/fixtures.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/zoom.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select/tests/fixtures.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop/tests/fixtures.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/keyboard/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/models.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/readouts.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/readouts/status.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu/tests.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/context_menu/tests/fixtures.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/tests.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/keyboard/tests/fixtures.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/navigation.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/keyboard/navigation/tests.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/keyboard/navigation/tests/fixtures.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/projection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all/tests.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/select_all/tests/fixtures.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete/tests.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/delete/tests/fixtures.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/tests.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming/tests.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming/tests/fixtures.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests/fixtures.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests/fixtures.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs"),
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
        "proof_collection_keyboard_apply_delete(",
        "proof_collection_keyboard_begin_rename(",
        "proof_collection_keyboard_apply_select_all(",
        "proof_collection_keyboard_apply_duplicate(",
        "proof_collection_keyboard_apply_navigation(",
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
        "mod actions;",
        "proof_collection_keyboard_apply_delete(",
        "proof_collection_keyboard_begin_rename(",
        "proof_collection_keyboard_apply_select_all(",
        "proof_collection_keyboard_apply_duplicate(",
        "proof_collection_keyboard_apply_navigation(",
    ] {
        assert!(
            keyboard_source.contains(needle),
            "collection keyboard owner should route app-state writes through keyboard/actions.rs; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_delete_status(",
        "proof_collection_duplicate_status(",
        "proof_collection_select_all_status(",
        "proof_collection_rename_ready_status(",
        "host.update_model(&models.assets",
        "host.update_model(&models.selection",
        "host.update_model(&models.keyboard",
        "host.update_model(&models.command_status",
        "host.notify(acx);",
    ] {
        assert!(
            !keyboard_source.contains(needle),
            "collection keyboard owner should delegate app-state mutation/status writes to keyboard/actions.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_keyboard_apply_delete(",
        "pub(super) fn proof_collection_keyboard_begin_rename(",
        "pub(super) fn proof_collection_keyboard_apply_select_all(",
        "pub(super) fn proof_collection_keyboard_apply_duplicate(",
        "pub(super) fn proof_collection_keyboard_apply_navigation(",
        "proof_collection_delete_status(&delete.deleted_assets)",
        "proof_collection_duplicate_status(&duplicate.duplicated_assets)",
        "proof_collection_select_all_status(next_selection.selected_count())",
        "proof_collection_rename_ready_status(",
        "host.update_model(&models.assets",
        "host.update_model(&models.selection",
        "host.update_model(&models.keyboard",
        "host.update_model(&models.command_status",
        "host.notify(acx);",
    ] {
        assert!(
            keyboard_actions_source.contains(needle),
            "collection keyboard actions owner should keep app-state mutation explicit; missing `{needle}`"
        );
    }
    for needle in [
        "cx.key_on_key_down_for(",
        "proof_collection_delete_key_matches(",
        "proof_collection_rename_shortcut_matches(",
        "proof_collection_select_all_shortcut_matches(",
        "proof_collection_duplicate_shortcut_matches(",
        "proof_collection_keyboard_selection(",
        "proof_collection_assets_in_visible_order(",
        "host.models_mut().read(",
    ] {
        assert!(
            !keyboard_actions_source.contains(needle),
            "collection keyboard actions owner should not take key matching, snapshot reads, or selection derivation policy; unexpected `{needle}`"
        );
    }

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
