#[test]
fn imui_editor_proof_demo_routes_collection_proof_through_demo_local_module() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let box_select_source = include_str!("../src/imui_editor_proof_demo/collection/box_select.rs");
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs");
    let drag_drop_source = include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs");
    let geometry_source = include_str!("../src/imui_editor_proof_demo/collection/geometry.rs");
    let keyboard_source = include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs");
    let models_source = include_str!("../src/imui_editor_proof_demo/collection/models.rs");
    let rename_source = include_str!("../src/imui_editor_proof_demo/collection/rename.rs");
    let selection_source = include_str!("../src/imui_editor_proof_demo/collection/selection.rs");
    let selection_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs");

    for needle in [
        "mod collection;",
        "collection::render_collection_first_asset_browser_proof(ui);",
        "collection::authoring_parity_collection_assets()",
    ] {
        assert!(
            demo_source.contains(needle),
            "imui_editor_proof_demo should keep the collection proof routed through the demo-local module; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_assets_in_visible_order(",
        "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "struct ProofCollectionAsset {",
        "fn proof_collection_drag_rect_normalizes_drag_direction()",
    ] {
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not keep the collection implementation inline after modularization; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "pub(super) fn render_collection_first_asset_browser_proof(",
        "ui: &mut ImUi<'_, '_, KernelApp>",
        "mod asset_grid;",
        "mod browser_scope;",
        "mod box_select;",
        "mod command_buttons;",
        "mod context_menu;",
        "mod drag_drop;",
        "mod geometry;",
        "mod keyboard;",
        "mod models;",
        "mod rename;",
        "mod selection;",
    ] {
        assert!(
            collection_source.contains(needle),
            "the demo-local collection module should keep the modularized implementation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeModels",
        "pub(super) struct ProofCollectionBrowserScopeState",
        "pub(super) fn render_collection_browser_scope(",
        "ui.child_region_with_options(",
        "install_collection_keyboard_handler(",
        "cx.pointer_region_on_wheel(",
        "cx.pointer_region_on_pointer_down(",
        "render_collection_asset_grid(",
        "imui-editor-proof.authoring.imui.collection.box-select.scope",
    ] {
        assert!(
            browser_scope_source.contains(needle),
            "the demo-local collection browser-scope owner should keep child-region pointer runtime explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionAssetGridModels",
        "pub(super) struct ProofCollectionAssetGridState",
        "pub(super) fn render_collection_asset_grid(",
        "ui.grid_with_options(",
        "TextField::new(",
        "drag_preview_ghost_with_options(",
        "ProofCollectionRenderedItem {",
    ] {
        assert!(
            asset_grid_source.contains(needle),
            "the demo-local collection asset-grid owner should keep tile-grid interaction explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionBoxSelectSession",
        "pub(super) struct ProofCollectionBoxSelectState",
        "pub(super) struct ProofCollectionRenderedItem",
        "pub(super) fn proof_collection_box_select_selection(",
        "pub(super) fn proof_collection_box_select_active_rect(",
    ] {
        assert!(
            box_select_source.contains(needle),
            "the demo-local collection box-select owner should keep marquee selection state explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionCommandButtonModels",
        "pub(super) struct ProofCollectionCommandButtonState",
        "pub(super) fn render_collection_command_buttons(",
        "proof_collection_set_command_status(",
    ] {
        assert!(
            command_buttons_source.contains(needle),
            "the demo-local collection command-buttons owner should keep explicit command button routing separate; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionContextMenuModels",
        "pub(super) fn render_collection_context_menu(",
        "PROOF_COLLECTION_CONTEXT_MENU_POPUP_ID",
        "ui.begin_popup_menu(",
        "kit::MenuItemOptions {",
    ] {
        assert!(
            context_menu_source.contains(needle),
            "the demo-local collection context-menu owner should keep popup workflow explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionDragPayload",
        "pub(super) fn proof_collection_drag_payload_for_asset(",
        "pub(super) fn proof_collection_drag_preview_title(",
        "pub(super) fn proof_collection_drag_preview_subtitle(",
        "pub(super) fn proof_collection_drop_status(",
    ] {
        assert!(
            drag_drop_source.contains(needle),
            "the demo-local collection drag/drop owner should keep payload and status projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "#[cfg(test)]",
        "fn proof_collection_drag_rect_normalizes_drag_direction() {",
    ] {
        assert!(
            geometry_source.contains(needle),
            "the demo-local collection geometry owner should keep the pure geometry test floor explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionKeyboardHandlerModels",
        "pub(super) fn install_collection_keyboard_handler(",
        "cx.key_on_key_down_for(",
        "proof_collection_keyboard_selection(",
    ] {
        assert!(
            keyboard_source.contains(needle),
            "the demo-local collection keyboard owner should keep scope keyboard dispatch explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn authoring_parity_collection_selection_model<H: UiHost>(",
        "pub(super) fn authoring_parity_collection_assets_model<H: UiHost>(",
        "pub(super) fn authoring_parity_collection_scroll_handle<H: UiHost>(",
    ] {
        assert!(
            models_source.contains(needle),
            "the demo-local collection models owner should keep state slot registration explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionRenameSession",
        "pub(super) fn proof_collection_begin_rename_session(",
        "pub(super) fn proof_collection_commit_rename(",
        "pub(super) fn proof_collection_sync_inline_rename_focus<",
    ] {
        assert!(
            rename_source.contains(needle),
            "the demo-local collection rename owner should keep inline rename workflow state explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod commands;",
        "pub(super) use commands::{",
        "pub(super) struct ProofCollectionKeyboardState",
        "pub(super) fn proof_collection_assets_in_visible_order(",
        "pub(super) fn proof_collection_keyboard_selection(",
    ] {
        assert!(
            selection_source.contains(needle),
            "the demo-local collection selection owner should keep pure selection state and command delegation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionDeleteResult",
        "pub(in super::super) struct ProofCollectionDuplicateResult",
        "pub(in super::super) fn proof_collection_delete_selection(",
        "pub(in super::super) fn proof_collection_duplicate_selection(",
        "fn proof_collection_unique_copy_text(",
    ] {
        assert!(
            selection_commands_source.contains(needle),
            "the demo-local collection selection command owner should keep duplicate/delete state transitions explicit; missing `{needle}`"
        );
    }
}
