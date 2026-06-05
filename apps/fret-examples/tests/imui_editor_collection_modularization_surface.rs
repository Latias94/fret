#[test]
fn imui_editor_proof_demo_routes_collection_proof_through_demo_local_module() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let box_select_source = include_str!("../src/imui_editor_proof_demo/collection/box_select.rs");
    let context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs");
    let drag_drop_source = include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs");
    let geometry_source = include_str!("../src/imui_editor_proof_demo/collection/geometry.rs");
    let models_source = include_str!("../src/imui_editor_proof_demo/collection/models.rs");
    let rename_source = include_str!("../src/imui_editor_proof_demo/collection/rename.rs");
    let selection_source = include_str!("../src/imui_editor_proof_demo/collection/selection.rs");

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
        "mod box_select;",
        "mod context_menu;",
        "mod drag_drop;",
        "mod geometry;",
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
        "pub(super) struct ProofCollectionKeyboardState",
        "pub(super) fn proof_collection_assets_in_visible_order(",
        "pub(super) fn proof_collection_keyboard_selection(",
        "pub(super) fn proof_collection_duplicate_selection(",
    ] {
        assert!(
            selection_source.contains(needle),
            "the demo-local collection selection owner should keep pure selection state transitions explicit; missing `{needle}`"
        );
    }
}
