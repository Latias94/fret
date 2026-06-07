#[test]
fn imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit() {
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let asset_grid_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/chrome.rs");
    let asset_grid_inline_rename_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs");
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let browser_scope_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs");
    let browser_input_runtime_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs");
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let command_buttons_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/chrome.rs");
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/chrome.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all.rs"),
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
        "Some(\"Primary+D\")",
        "shortcut: shortcut.map(Arc::from)",
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
        "mod chrome;",
        "let duplicate_selected = ui.button_with_options(",
        "collection_duplicate_selected_label()",
        "collection_duplicate_selected_button_options(!state.selection.is_empty())",
        "proof_collection_duplicate_selection(",
        "collection_rename_active_label()",
        "collection_rename_active_button_options(state.rename_ready_session.is_some())",
        "proof_collection_begin_inline_rename_in_app(",
        "let delete_selected = ui.button_with_options(",
        "collection_delete_selected_label()",
        "collection_delete_selected_button_options(!state.selection.is_empty())",
        "proof_collection_delete_selection(",
        "proof_collection_set_command_status(",
    ] {
        assert!(
            command_buttons_source.contains(needle),
            "collection command-buttons owner should route explicit command buttons through app-owned state transitions; missing `{needle}`"
        );
    }
    for needle in [
        "kit::ButtonOptions",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"Delete selected assets\"",
        "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
    ] {
        assert!(
            !command_buttons_source.contains(needle),
            "collection command-buttons owner should delegate button chrome construction to command_buttons/chrome.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn collection_duplicate_selected_label() -> &'static str",
        "pub(super) fn collection_rename_active_label() -> &'static str",
        "pub(super) fn collection_delete_selected_label() -> &'static str",
        "pub(super) fn collection_duplicate_selected_button_options(enabled: bool) -> kit::ButtonOptions",
        "pub(super) fn collection_rename_active_button_options(enabled: bool) -> kit::ButtonOptions",
        "pub(super) fn collection_delete_selected_button_options(enabled: bool) -> kit::ButtonOptions",
        "kit::ButtonOptions",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"Delete selected assets\"",
        "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
    ] {
        assert!(
            command_buttons_chrome_source.contains(needle),
            "collection command-buttons chrome owner should keep option/test-id construction explicit; missing `{needle}`"
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
        "mod chrome;",
        "mod inline_rename;",
        "collection_asset_grid_options(",
        "collection_asset_selectable_options(",
        "collection_asset_ghost_options(",
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
        "kit::GridOptions",
        "kit::VerticalOptions",
        "kit::SelectableOptions",
        "DragPreviewGhostOptions",
        "\"imui-editor-proof.authoring.imui.collection.grid\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.select\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.ghost\"",
    ] {
        assert!(
            !asset_grid_source.contains(needle),
            "collection asset-grid owner should delegate option/test-id construction to the chrome owner; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn collection_asset_grid_options(",
        "pub(super) fn collection_asset_tile_options(",
        "pub(super) fn collection_asset_selectable_options(",
        "pub(super) fn collection_asset_ghost_id(",
        "pub(super) fn collection_asset_ghost_options(",
        "kit::GridOptions",
        "kit::VerticalOptions",
        "kit::SelectableOptions",
        "DragPreviewGhostOptions",
        "\"imui-editor-proof.authoring.imui.collection.grid\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.select\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.ghost\"",
    ] {
        assert!(
            asset_grid_chrome_source.contains(needle),
            "collection asset-grid chrome owner should keep option/test-id construction explicit; missing `{needle}`"
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
        "mod chrome;",
        "collection_browser_child_region_options(",
        "collection_browser_box_select_marquee(",
        "collection_browser_box_select_scope_id()",
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
        "pub(super) fn collection_browser_child_region_options(",
        "pub(super) fn collection_browser_box_select_marquee(",
        "kit::ChildRegionOptions",
        "kit::ScrollOptions",
        "\"imui-editor-proof.authoring.imui.collection.browser\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.viewport\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.content\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
        ".border_1()",
    ] {
        assert!(
            browser_scope_chrome_source.contains(needle),
            "collection browser-scope chrome owner should keep option/test-id and marquee chrome construction explicit; missing `{needle}`"
        );
    }
    for needle in [
        "kit::ChildRegionOptions",
        "kit::ScrollOptions",
        "\"imui-editor-proof.authoring.imui.collection.browser\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.viewport\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.content\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
        ".border_1()",
    ] {
        assert!(
            !browser_scope_source.contains(needle),
            "collection browser-scope owner should delegate option/test-id and marquee chrome construction to the chrome owner; unexpected `{needle}`"
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
