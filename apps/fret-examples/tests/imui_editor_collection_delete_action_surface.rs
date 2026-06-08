#[test]
fn imui_editor_proof_demo_keeps_collection_delete_action_app_owned_and_explicit() {
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
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests.rs"
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
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/chrome.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/navigation.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/projection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete/tests.rs"),
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
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs"),
    );

    for needle in [
        "struct ProofCollectionDeleteResult {",
        "fn proof_collection_assets_line(",
        "fn proof_collection_delete_key_matches(",
        "fn proof_collection_delete_selection(",
        "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
        "proof_collection_delete_selection_picks_previous_visible_item_at_end",
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
