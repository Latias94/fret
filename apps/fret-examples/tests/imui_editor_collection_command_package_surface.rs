#[test]
fn imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit() {
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let asset_grid_tile_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/tile.rs");
    let asset_grid_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/chrome.rs");
    let asset_grid_inline_rename_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs");
    let asset_grid_inline_rename_actions_source = include_str!(
        "../src/imui_editor_proof_demo/collection/asset_grid/inline_rename/actions.rs"
    );
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let browser_scope_asset_grid_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/asset_grid.rs");
    let browser_scope_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs");
    let browser_input_runtime_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs");
    let browser_input_box_select_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select.rs"
    );
    let browser_input_box_select_session_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session.rs"
    );
    let browser_input_box_select_session_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests.rs"
    );
    let browser_input_context_menu_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu.rs"
    );
    let browser_input_zoom_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/zoom.rs"
    );
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let command_buttons_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs");
    let command_buttons_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/chrome.rs");
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/tile.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/asset_grid/inline_rename/actions.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/geometry.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom.rs"),
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
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete/tests.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming.rs"
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
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs"),
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
        "mod actions;",
        "mod chrome;",
        "let duplicate_selected = ui.button_with_options(",
        "collection_duplicate_selected_label()",
        "collection_duplicate_selected_button_options(!state.selection.is_empty())",
        "proof_collection_duplicate_selection(",
        "proof_collection_command_button_apply_duplicate(",
        "collection_rename_active_label()",
        "collection_rename_active_button_options(state.rename_ready_session.is_some())",
        "proof_collection_command_button_begin_rename(",
        "let delete_selected = ui.button_with_options(",
        "collection_delete_selected_label()",
        "collection_delete_selected_button_options(!state.selection.is_empty())",
        "proof_collection_delete_selection(",
        "proof_collection_command_button_apply_delete(",
    ] {
        assert!(
            command_buttons_source.contains(needle),
            "collection command-buttons owner should route explicit command buttons through app-owned state transitions; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_duplicate_status(",
        "proof_collection_delete_status(",
        "proof_collection_begin_inline_rename_in_app(",
        "proof_collection_set_command_status(",
        "models_mut().update",
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
        "pub(super) fn proof_collection_command_button_apply_duplicate(",
        "pub(super) fn proof_collection_command_button_begin_rename(",
        "pub(super) fn proof_collection_command_button_apply_delete(",
        "proof_collection_duplicate_status(&duplicate.duplicated_assets)",
        "proof_collection_delete_status(&delete.deleted_assets)",
        "proof_collection_begin_inline_rename_in_app(",
        "app.models_mut().update(&models.assets",
        "app.models_mut().update(&models.selection",
        "app.models_mut().update(&models.keyboard",
        "proof_collection_set_command_status(",
    ] {
        assert!(
            command_buttons_actions_source.contains(needle),
            "collection command-buttons actions owner should keep button-triggered state writes explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ui.button_with_options(",
        "collection_duplicate_selected_label()",
        "collection_duplicate_selected_button_options(",
        "collection_rename_active_label()",
        "collection_rename_active_button_options(",
        "collection_delete_selected_label()",
        "collection_delete_selected_button_options(",
        "proof_collection_duplicate_selection(",
        "proof_collection_delete_selection(",
    ] {
        assert!(
            !command_buttons_actions_source.contains(needle),
            "collection command-buttons actions owner should not take button rendering or selection policy; unexpected `{needle}`"
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
        "mod tile;",
        "render_collection_asset_tile(",
        "mod chrome;",
        "mod inline_rename;",
        "collection_asset_grid_options(",
        "ui.grid_with_options(",
    ] {
        assert!(
            asset_grid_source.contains(needle),
            "collection asset-grid owner should keep grid entry and route tile rendering through the tile owner; missing `{needle}`"
        );
    }
    for needle in [
        "collection_asset_selectable_options(",
        "collection_asset_ghost_options(",
        "ui.multi_selectable_with_options(",
        "render_collection_inline_rename_field(",
        "drag_preview_ghost_with_options(",
    ] {
        assert!(
            asset_grid_tile_source.contains(needle),
            "collection asset-grid tile owner should keep tile-grid interaction through app-owned state transitions; missing `{needle}`"
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
        "ui.multi_selectable_with_options(",
        "drag_preview_ghost_with_options(",
        "render_collection_inline_rename_field(",
    ] {
        assert!(
            !asset_grid_source.contains(needle),
            "collection asset-grid owner should delegate option/test-id construction to chrome and tile interaction to asset_grid/tile.rs; unexpected `{needle}`"
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
        "mod actions;",
        "proof_collection_inline_rename_apply_outcome(",
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
    for needle in [
        "pub(super) struct ProofCollectionInlineRenameOutcomeModels",
        "pub(super) fn proof_collection_inline_rename_apply_outcome(",
        "proof_collection_commit_rename(",
        "host.update_model(&models.rename_status",
        "proof_collection_restore_focus_after_inline_rename(",
    ] {
        assert!(
            asset_grid_inline_rename_actions_source.contains(needle),
            "collection asset-grid inline-rename actions owner should keep outcome model writes explicit; missing `{needle}`"
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
        "mod asset_grid;",
        "mod chrome;",
        "render_collection_browser_scope_asset_grid(",
        "collection_browser_child_region_options(",
        "collection_browser_box_select_marquee(",
        "collection_browser_box_select_scope_id()",
        "proof_collection_browser_scope_pointer_props()",
        "install_collection_browser_scope_input_runtime(",
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
        "render_collection_asset_grid(",
        "ProofCollectionAssetGridModels {",
        "ProofCollectionAssetGridState {",
    ] {
        assert!(
            !browser_scope_source.contains(needle),
            "collection browser-scope owner should delegate chrome construction to browser_scope/chrome.rs and asset-grid mounting to browser_scope/asset_grid.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeAssetGridModels",
        "pub(super) struct ProofCollectionBrowserScopeAssetGridState",
        "pub(super) fn render_collection_browser_scope_asset_grid(",
        "render_collection_asset_grid(",
        "ProofCollectionAssetGridModels {",
        "ProofCollectionAssetGridState {",
        ".w_full()",
        ".into_element(cx)",
    ] {
        assert!(
            browser_scope_asset_grid_source.contains(needle),
            "collection browser-scope asset-grid owner should keep grid mounting explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ui.child_region_with_options(",
        "proof_collection_browser_scope_pointer_props()",
        "install_collection_browser_scope_input_runtime(",
        "collection_browser_box_select_marquee(",
        "proof_collection_box_select_active_rect(",
        "kit::ChildRegionOptions",
        "cx.pointer_region(",
    ] {
        assert!(
            !browser_scope_asset_grid_source.contains(needle),
            "collection browser-scope asset-grid owner should not take child-region, pointer runtime, or marquee chrome responsibilities; unexpected `{needle}`"
        );
    }
    for needle in [
        "mod box_select;",
        "mod context_menu;",
        "mod zoom;",
        "use box_select::{",
        "install_collection_browser_scope_box_select_runtime,",
        "use context_menu::publish_collection_browser_scope_context_menu_anchor;",
        "use zoom::install_collection_browser_scope_zoom_runtime;",
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "pub(super) fn proof_collection_browser_scope_pointer_props()",
        "pub(super) fn install_collection_browser_scope_input_runtime(",
        "publish_collection_browser_scope_context_menu_anchor(",
        "install_collection_browser_scope_zoom_runtime(",
        "install_collection_browser_scope_box_select_runtime(",
        "ProofCollectionBrowserScopeBoxSelectRuntimeModels {",
        "ProofCollectionBrowserScopeBoxSelectRuntimeState {",
        "context_menu_anchor_model_for_up",
    ] {
        assert!(
            browser_input_runtime_source.contains(needle),
            "collection browser input runtime owner should keep wheel/context/box-select handlers explicit; missing `{needle}`"
        );
    }
    for needle in [
        "cx.pointer_region_on_wheel(",
        "proof_collection_zoom_request(",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
        "up.down_hit_pressable_target.is_some()",
        "up.position_window.unwrap_or(up.position)",
        "*state = Some(position);",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
        "host.request_focus(acx.target);",
        "ProofCollectionBoxSelectSession {",
        "host.capture_pointer();",
        "proof_collection_box_select_selection(",
        "state.clear();",
        "host.release_pointer_capture();",
    ] {
        assert!(
            !browser_input_runtime_source.contains(needle),
            "collection browser input runtime owner should route wheel/context/box-select child behavior through child runtime owners; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "mod session;",
        "use session::{",
        "cx.pointer_region_on_pointer_down(",
        "host.request_focus(acx.target);",
        "proof_collection_browser_scope_box_select_can_start_from_down(",
        "proof_collection_browser_scope_box_select_session_from_down(",
        "host.capture_pointer();",
        "cx.pointer_region_on_pointer_move(",
        "proof_collection_browser_scope_box_select_session_for_move(",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "cx.pointer_region_on_pointer_up(",
        "before_box_select_pointer_up(host, acx, &up)",
        "proof_collection_browser_scope_box_select_session_for_up(",
        "host.release_pointer_capture();",
        "state.clear();",
        "cx.pointer_region_on_pointer_cancel(",
        "proof_collection_browser_scope_box_select_cancel_pointer(",
        "proof_collection_box_select_selection(",
        "state.active_id = next_selection.first_selected().cloned();",
    ] {
        assert!(
            browser_input_box_select_runtime_source.contains(needle),
            "collection browser input box-select runtime owner should keep pointer event wiring and selection publication explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ProofCollectionBoxSelectSession {",
        "fn proof_collection_browser_scope_box_select_can_start_from_down(",
        "fn proof_collection_browser_scope_box_select_session_from_down(",
        "fn proof_collection_browser_scope_box_select_update_session_position(",
        "fn proof_collection_browser_scope_box_select_session_for_move(",
        "fn proof_collection_browser_scope_box_select_session_for_up(",
        "fn proof_collection_browser_scope_box_select_cancel_pointer(",
        "box_select_down_arms_left_background_session",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
    ] {
        assert!(
            !browser_input_box_select_runtime_source.contains(needle),
            "collection browser input box-select runtime owner should route pure pointer session transitions through box_select/session.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_browser_scope_box_select_can_start_from_down(",
        "pub(super) fn proof_collection_browser_scope_box_select_session_from_down(",
        "fn proof_collection_browser_scope_box_select_update_session_position(",
        "pub(super) fn proof_collection_browser_scope_box_select_session_for_move(",
        "pub(super) fn proof_collection_browser_scope_box_select_session_for_up(",
        "pub(super) fn proof_collection_browser_scope_box_select_cancel_pointer(",
        "proof_collection_drag_threshold_met(",
        "ProofCollectionBoxSelectSession {",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            browser_input_box_select_session_source.contains(needle),
            "collection browser input box-select session owner should keep pure pointer session transitions explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "BeforeCollectionBrowserScopeBoxSelectPointerUp",
        "cx.pointer_region_on_pointer_down(",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "proof_collection_box_select_selection(",
        "state.active_id = next_selection.first_selected().cloned();",
        "fn pointer_down(",
        "fn pointer_move(",
        "fn pointer_up(",
        "fn pointer_cancel(",
        "box_select_down_arms_left_background_session",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
    ] {
        assert!(
            !browser_input_box_select_session_source.contains(needle),
            "collection browser input box-select session owner should not take runtime event/model publication; unexpected `{needle}`"
        );
    }
    for needle in [
        "fn pointer_down(",
        "fn pointer_move(",
        "fn pointer_up(",
        "fn pointer_cancel(",
        "fn session(pointer_id: PointerId) -> ProofCollectionBoxSelectSession",
        "box_select_down_arms_left_background_session",
        "box_select_down_ignores_non_left_or_pressable_origin",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_move_ignores_released_left_button",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
    ] {
        assert!(
            browser_input_box_select_session_tests_source.contains(needle),
            "collection browser input box-select session tests owner should keep pointer fixtures and behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "BeforeCollectionBrowserScopeBoxSelectPointerUp",
        "cx.pointer_region_on_pointer_down(",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "proof_collection_box_select_selection(",
        "state.active_id = next_selection.first_selected().cloned();",
    ] {
        assert!(
            !browser_input_box_select_session_tests_source.contains(needle),
            "collection browser input box-select session tests owner should not take runtime event/model publication; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "install_collection_keyboard_handler(",
        "install_collection_browser_scope_zoom_runtime(",
        "pub(super) fn publish_collection_browser_scope_context_menu_anchor(",
        "proof_collection_zoom_request(",
        "up.down_hit_pressable_target.is_some()",
        "up.position_window.unwrap_or(up.position)",
        "*state = Some(position);",
    ] {
        assert!(
            !browser_input_box_select_runtime_source.contains(needle),
            "collection browser input box-select runtime owner should not take parent keyboard/zoom/context-menu responsibilities; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_browser_scope_context_menu_anchor_from_up(",
        "up.button != MouseButton::Right || !up.is_click",
        "up.down_hit_pressable_target.is_some()",
        "up.down_hit_pressable_target_in_descendant_subtree",
        "Some(up.position_window.unwrap_or(up.position))",
        "pub(super) fn publish_collection_browser_scope_context_menu_anchor(",
        "host.request_focus(acx.target);",
        "host.update_model(context_menu_anchor_model",
        "*state = Some(position);",
        "host.notify(acx);",
        "context_menu_anchor_prefers_window_position",
        "context_menu_anchor_ignores_direct_pressable_clicks",
        "context_menu_anchor_ignores_pressable_descendant_clicks",
    ] {
        assert!(
            browser_input_context_menu_runtime_source.contains(needle),
            "collection browser input context-menu runtime owner should keep right-click anchor publishing explicit; missing `{needle}`"
        );
    }
    for needle in [
        "install_collection_keyboard_handler(",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
        "ProofCollectionBoxSelectSession",
        "proof_collection_box_select_selection(",
        "proof_collection_zoom_request(",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
    ] {
        assert!(
            !browser_input_context_menu_runtime_source.contains(needle),
            "collection browser input context-menu runtime owner should not take keyboard/box-select/zoom runtime; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn install_collection_browser_scope_zoom_runtime(",
        "cx.pointer_region_on_wheel(",
        "proof_collection_zoom_request(",
        "collection_scroll_handle.offset()",
        "wheel.position_local",
        "wheel.delta",
        "wheel.modifiers",
        "host.update_model(&collection_zoom_model",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
        "host.notify(acx);",
    ] {
        assert!(
            browser_input_zoom_runtime_source.contains(needle),
            "collection browser input zoom runtime owner should keep Primary+Wheel zoom explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "install_collection_keyboard_handler(",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
        "proof_collection_box_select_selection(",
        "context_menu_anchor_model_for_up",
    ] {
        assert!(
            !browser_input_zoom_runtime_source.contains(needle),
            "collection browser input zoom runtime owner should not take keyboard/context/box-select runtime; unexpected `{needle}`"
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
