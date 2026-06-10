pub(super) fn assert_runtime_state_owner_split(
    collection_source: &str,
    runtime_state_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionRuntimeState",
        "pub(super) struct ProofCollectionRuntimeModels",
        "pub(super) struct ProofCollectionRuntimeSnapshot",
        "pub(super) fn rename_session(&self) -> Option<&ProofCollectionRenameSession>",
        "pub(super) fn proof_collection_runtime_state(",
        "selection: authoring_parity_collection_selection_model(ui.cx_mut())",
        "assets: authoring_parity_collection_assets_model(ui.cx_mut())",
        "reverse_order: authoring_parity_collection_reverse_order_model(ui.cx_mut())",
        "box_select: authoring_parity_collection_box_select_model(ui.cx_mut())",
        "keyboard: authoring_parity_collection_keyboard_model(ui.cx_mut())",
        "zoom: authoring_parity_collection_zoom_model(ui.cx_mut())",
        "context_menu_anchor: authoring_parity_collection_context_menu_anchor_model(ui.cx_mut())",
        "rename_session: authoring_parity_collection_rename_session_model(ui.cx_mut())",
        "rename_draft: authoring_parity_collection_rename_draft_model(ui.cx_mut())",
        "rename_focus_pending: authoring_parity_collection_rename_focus_pending_model(ui.cx_mut())",
        "active_focus_target: authoring_parity_collection_active_focus_target_model(ui.cx_mut())",
        "rename_status: authoring_parity_collection_rename_status_model(ui.cx_mut())",
        "command_status: authoring_parity_collection_command_status_model(ui.cx_mut())",
        "scroll: authoring_parity_collection_scroll_handle(ui.cx_mut())",
        "fn proof_collection_runtime_snapshot(",
        "selector_model_paint(&models.assets, |state| state.clone())",
        "selector_model_paint(&models.selection, |state| state)",
        "selector_model_paint(&models.rename_status, |state| state.clone())",
        "proof_collection_layout_metrics(models.scroll.viewport_size().width, tile_extent)",
    ] {
        assert!(
            runtime_state_source.contains(needle),
            "the demo-local collection runtime-state owner should keep model handles, selector snapshots, and layout projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "authoring_parity_collection_selection_model(ui.cx_mut())",
        "authoring_parity_collection_assets_model(ui.cx_mut())",
        "authoring_parity_collection_reverse_order_model(ui.cx_mut())",
        "authoring_parity_collection_box_select_model(ui.cx_mut())",
        "authoring_parity_collection_keyboard_model(ui.cx_mut())",
        "authoring_parity_collection_zoom_model(ui.cx_mut())",
        "authoring_parity_collection_context_menu_anchor_model(ui.cx_mut())",
        "authoring_parity_collection_rename_session_model(ui.cx_mut())",
        "authoring_parity_collection_rename_draft_model(ui.cx_mut())",
        "authoring_parity_collection_rename_focus_pending_model(ui.cx_mut())",
        "authoring_parity_collection_active_focus_target_model(ui.cx_mut())",
        "authoring_parity_collection_rename_status_model(ui.cx_mut())",
        "authoring_parity_collection_command_status_model(ui.cx_mut())",
        "authoring_parity_collection_scroll_handle(ui.cx_mut())",
        "selector_model_paint(",
        "proof_collection_layout_metrics(",
        "use fret::advanced::view::AppRenderDataExt as _;",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route runtime model/snapshot reads through collection/runtime_state.rs; unexpected `{needle}`"
        );
    }
}
