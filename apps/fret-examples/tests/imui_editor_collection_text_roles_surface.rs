#[test]
fn imui_editor_proof_collection_fixed_text_uses_shared_roles() {
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/tile.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/asset_grid/inline_rename/actions.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/metadata.rs"),
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
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/zoom.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/import_target.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/status_readouts.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs"),
    );

    for needle in [
        "proof_compact_readout_element",
        "proof_section_chrome_label",
        "fn proof_collection_readout_text(",
        "fn render_collection_header(",
        "fn proof_collection_section_label(",
        "proof_collection_section_label(\n        ui,\n        \"Collection-first asset browser proof\",",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_assets_line(state.assets),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_visible_order_line(state.assets),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_select_all_line(),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_rename_line(),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_command_status_line(state.command_status),",
        "ui.text_wrapped(\n        \"Inline rename stays app-owned: Enter commits; Escape or blur cancels without widening shared IMUI helpers.\",",
        "proof_collection_readout_text(\n        ui,\n        format!(\"{} | {} KiB\", asset.kind, asset.size_kib),",
        "\"imui-editor-proof.authoring.imui.collection.asset.metadata\"",
        "proof_collection_readout_text(\n        ui,\n        asset.path.clone(),",
        "\"imui-editor-proof.authoring.imui.collection.asset.path\"",
        "proof_collection_readout_text(\n            ui,\n            format!(",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.selection-readout\"",
        "proof_collection_readout_text(\n        ui,\n        visible_collection_status,",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            source.contains(needle),
            "collection proof fixed text should use shared text-role helpers; missing `{needle}`"
        );
    }

    for needle in [
        "ui.text(\"Collection-first asset browser proof\");",
        "ui.text(proof_collection_assets_line(&collection_assets));",
        "ui.text(proof_collection_visible_order_line(&collection_assets));",
        "ui.text(proof_collection_select_all_line());",
        "ui.text(proof_collection_rename_line());",
        "ui.text(proof_collection_command_status_line(",
        "ui.text(\n        \"Inline rename stays app-owned: Enter commits; Escape or blur cancels without widening shared IMUI helpers.\",",
        "ui.text(format!(\"{} | {} KiB\",",
        "ui.text(asset.path.clone());",
        "ui.text(format!(\n            \"Selection: {} item(s)\",",
        "ui.text(visible_collection_status);",
    ] {
        assert!(
            !source.contains(needle),
            "collection proof fixed text should not use bare IMUI text; unexpected `{needle}`"
        );
    }
}
