pub(super) fn assert_asset_grid_owner_split(
    asset_grid_source: &str,
    asset_grid_tile_source: &str,
    asset_grid_actions_source: &str,
    asset_grid_chrome_source: &str,
    asset_grid_inline_rename_source: &str,
    asset_grid_inline_rename_actions_source: &str,
    asset_grid_metadata_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionAssetGridModels",
        "pub(super) struct ProofCollectionAssetGridState",
        "pub(super) fn render_collection_asset_grid(",
        "ui.grid_with_options(",
        "mod actions;",
        "mod chrome;",
        "mod inline_rename;",
        "mod metadata;",
        "mod tile;",
        "render_collection_asset_tile(",
        "collection_asset_grid_options(",
    ] {
        assert!(
            asset_grid_source.contains(needle),
            "the demo-local collection asset-grid owner should keep grid entry and route tile rendering explicitly; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_asset_grid_publish_active_focus_target(",
        "proof_collection_asset_grid_activate_clicked_asset(",
        "proof_collection_asset_grid_apply_context_menu(",
        "collection_asset_tile_options(",
        "collection_asset_selectable_options(",
        "collection_asset_ghost_id(",
        "collection_asset_ghost_options(",
        "render_collection_inline_rename_field(",
        "render_collection_asset_metadata_readouts(",
        "drag_preview_ghost_with_options(",
        "ProofCollectionRenderedItem {",
    ] {
        assert!(
            asset_grid_tile_source.contains(needle),
            "the demo-local collection asset-grid tile owner should keep tile-grid interaction explicit; missing `{needle}`"
        );
    }
    for needle in [
        "models_mut().update",
        "kit::GridOptions",
        "kit::VerticalOptions",
        "kit::SelectableOptions",
        "DragPreviewGhostOptions",
        "\"imui-editor-proof.authoring.imui.collection.grid\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.select\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.ghost\"",
        "ui.multi_selectable_with_options(",
        "drag_preview_ghost_with_options(",
        "render_collection_inline_rename_field(",
        "render_collection_asset_metadata_readouts(",
        "ProofCollectionRenderedItem {",
    ] {
        assert!(
            !asset_grid_source.contains(needle),
            "the demo-local collection asset-grid owner should delegate option/test-id construction to asset_grid/chrome.rs and tile behavior to asset_grid/tile.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_asset_grid_publish_active_focus_target(",
        "pub(super) fn proof_collection_asset_grid_activate_clicked_asset(",
        "pub(super) fn proof_collection_asset_grid_apply_context_menu(",
        "app.models_mut().update(active_focus_target",
        "keyboard.active_id = Some(asset_id);",
        "app.models_mut().update(&models.selection",
        "app.models_mut().update(&models.keyboard",
        "app.models_mut()",
        ".update(&models.context_menu_anchor",
    ] {
        assert!(
            asset_grid_actions_source.contains(needle),
            "the demo-local collection asset-grid actions owner should keep tile-triggered model writes explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ui.grid_with_options(",
        "ui.multi_selectable_with_options(",
        "drag_preview_ghost_with_options(",
        "render_collection_inline_rename_field(",
        "render_collection_asset_metadata_readouts(",
        "proof_collection_drag_payload_for_asset(",
        "proof_collection_context_menu_selection(",
        "ProofCollectionRenderedItem {",
    ] {
        assert!(
            !asset_grid_actions_source.contains(needle),
            "the demo-local collection asset-grid actions owner should not take tile rendering, drag preview, metadata, or selection policy; unexpected `{needle}`"
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
        "fret_ui_kit::LayoutRefinement::default()",
        ".min_h(layout.tile_min_height)",
        "\"imui-editor-proof.authoring.imui.collection.grid\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.select\"",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.ghost\"",
    ] {
        assert!(
            asset_grid_chrome_source.contains(needle),
            "the demo-local collection asset-grid chrome owner should keep grid/tile/selectable/ghost options explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_inline_rename_field(",
        "mod actions;",
        "ProofCollectionInlineRenameOutcomeModels",
        "proof_collection_inline_rename_apply_outcome",
        "TextField::new(",
        ".on_outcome(Some(Arc::new(",
        "proof_collection_inline_rename_apply_outcome(",
        "TextFieldOptions {",
        "EditorTextSelectionBehavior::SelectAllOnFocus",
        "TextFieldBlurBehavior::Cancel",
        "\"imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline\"",
        "proof_collection_inline_rename_focus_state(",
        "proof_collection_sync_inline_rename_focus(",
    ] {
        assert!(
            asset_grid_inline_rename_source.contains(needle),
            "the demo-local collection asset-grid inline-rename owner should keep TextField workflow explicit; missing `{needle}`"
        );
    }
    for needle in [
        "host.update_model(",
        "proof_collection_commit_rename(",
        "proof_collection_rename_commit_status(",
        "proof_collection_rename_invalid_status(",
        "proof_collection_rename_cancel_status(",
        "proof_collection_restore_focus_after_inline_rename(",
    ] {
        assert!(
            !asset_grid_inline_rename_source.contains(needle),
            "the demo-local collection asset-grid inline-rename owner should route outcome model writes through inline_rename/actions.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionInlineRenameOutcomeModels",
        "pub(super) fn proof_collection_inline_rename_apply_outcome(",
        "fn proof_collection_inline_rename_apply_commit(",
        "fn proof_collection_inline_rename_apply_cancel(",
        "proof_collection_commit_rename(",
        "proof_collection_rename_commit_status(",
        "proof_collection_rename_invalid_status(",
        "proof_collection_rename_cancel_status(",
        "host.update_model(&models.assets",
        "host.update_model(&models.rename_status",
        "host.update_model(&models.rename_session",
        "host.update_model(&models.rename_focus_pending",
        "proof_collection_restore_focus_after_inline_rename(",
        "host.request_redraw(action_cx.window);",
    ] {
        assert!(
            asset_grid_inline_rename_actions_source.contains(needle),
            "the demo-local collection asset-grid inline-rename actions owner should keep outcome model writes explicit; missing `{needle}`"
        );
    }
    for needle in [
        "TextField::new(",
        "TextFieldOptions {",
        "EditorTextSelectionBehavior::SelectAllOnFocus",
        "TextFieldBlurBehavior::Cancel",
        "ui.text_wrapped(",
        "proof_collection_inline_rename_focus_state(",
        "proof_collection_sync_inline_rename_focus(",
    ] {
        assert!(
            !asset_grid_inline_rename_actions_source.contains(needle),
            "the demo-local collection asset-grid inline-rename actions owner should not take TextField rendering or focus timer sync; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_asset_metadata_readouts(",
        "proof_collection_readout_text(",
        "format!(\"{} | {} KiB\", asset.kind, asset.size_kib)",
        "\"imui-editor-proof.authoring.imui.collection.asset.metadata\"",
        "asset.path.clone()",
        "\"imui-editor-proof.authoring.imui.collection.asset.path\"",
    ] {
        assert!(
            asset_grid_metadata_source.contains(needle),
            "the demo-local collection asset-grid metadata owner should keep asset readout text explicit; missing `{needle}`"
        );
    }
}
