#[test]
fn imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit() {
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let asset_grid_inline_rename_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs");
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let browser_input_runtime_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs");
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs"),
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
        "struct ProofCollectionDuplicateResult {",
        "fn proof_collection_command_package_line() -> String {",
        "fn proof_collection_command_status_line(status: &str) -> String {",
        "fn proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_duplicate_selection(",
        "fn proof_collection_duplicate_status(",
        "fn proof_collection_begin_inline_rename_in_app(",
        "fn authoring_parity_collection_command_status_model<H: UiHost>(",
        "imui_editor_proof_demo.model.authoring_parity.collection_command_status",
        "\"Duplicate, delete, rename, and select-all stay inside one app-owned collection command package; duplicate/delete/rename now route across keyboard, explicit buttons, and context menu without widening shared IMUI helpers.\"",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected\"",
        "\"Command status: {status}\"",
        "proof_collection_duplicate_shortcut_matches(",
        "KeyCode::KeyD",
        "shortcut: Some(Arc::from(\"Primary+D\"))",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection command-package slice explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionCommandButtonModels",
        "pub(super) struct ProofCollectionCommandButtonState",
        "pub(super) fn render_collection_command_buttons(",
        "let duplicate_selected = ui.button_with_options(",
        "proof_collection_duplicate_selection(",
        "proof_collection_begin_inline_rename_in_app(",
        "let delete_selected = ui.button_with_options(",
        "proof_collection_delete_selection(",
        "proof_collection_set_command_status(",
    ] {
        assert!(
            command_buttons_source.contains(needle),
            "collection command-buttons owner should route explicit command buttons through app-owned state transitions; missing `{needle}`"
        );
    }
    assert!(
        !collection_source.contains("let duplicate_selected = ui.button_with_options("),
        "collection root should delegate explicit command buttons to the command-buttons owner"
    );

    for needle in [
        "pub(super) struct ProofCollectionAssetGridModels",
        "pub(super) struct ProofCollectionAssetGridState",
        "pub(super) fn render_collection_asset_grid(",
        "mod inline_rename;",
        "ui.grid_with_options(",
        "ui.multi_selectable_with_options(",
        "render_collection_inline_rename_field(",
        "drag_preview_ghost_with_options(",
    ] {
        assert!(
            asset_grid_source.contains(needle),
            "collection asset-grid owner should route tile-grid interaction through app-owned state transitions; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn render_collection_inline_rename_field(",
        "TextField::new(",
        "TextFieldOptions {",
        "EditorTextSelectionBehavior::SelectAllOnFocus",
        "TextFieldBlurBehavior::Cancel",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline\"",
    ] {
        assert!(
            asset_grid_inline_rename_source.contains(needle),
            "collection asset-grid inline-rename owner should keep TextField wiring explicit; missing `{needle}`"
        );
    }
    assert!(
        !collection_source.contains("ui.grid_with_options("),
        "collection root should delegate asset-grid rendering to the asset-grid owner"
    );

    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeModels",
        "pub(super) struct ProofCollectionBrowserScopeState",
        "pub(super) fn render_collection_browser_scope(",
        "ui.child_region_with_options(",
        "proof_collection_browser_scope_pointer_props()",
        "install_collection_browser_scope_input_runtime(",
        "render_collection_asset_grid(",
    ] {
        assert!(
            browser_scope_source.contains(needle),
            "collection browser-scope owner should route child-region pointer runtime through app-owned state transitions; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "pub(super) fn proof_collection_browser_scope_pointer_props()",
        "pub(super) fn install_collection_browser_scope_input_runtime(",
        "cx.pointer_region_on_wheel(",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
    ] {
        assert!(
            browser_input_runtime_source.contains(needle),
            "collection browser input runtime owner should keep wheel/context/box-select handlers explicit; missing `{needle}`"
        );
    }
    assert!(
        !collection_source.contains("ui.child_region_with_options("),
        "collection root should delegate browser child-region runtime to the browser-scope owner"
    );

    for needle in [
        "fret_ui_kit::imui::collection_command_package",
        "pub fn collection_command_package",
        "pub fn collection_duplicate_selected",
        "struct ImUiCollectionCommandPackage",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the command-package slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
