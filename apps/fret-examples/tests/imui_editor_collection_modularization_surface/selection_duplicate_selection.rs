pub(super) fn assert_selection_duplicate_selection_owner_split(
    selection_duplicate_selection_source: &str,
    selection_duplicate_selection_tests_source: &str,
    selection_duplicate_selection_tests_fixtures_source: &str,
) {
    for needle in [
        "pub(super) fn proof_collection_duplicate_selection_result(",
        "ProofCollectionDuplicateNameRegistry::from_assets(stored_assets)",
        "let mut duplicates_by_source = HashMap::<Arc<str>, ProofCollectionAsset>::new();",
        "name_registry.duplicate_id(asset.id.as_ref())",
        "name_registry.duplicate_label(asset.label.as_ref())",
        "name_registry.duplicate_path(asset.path.as_ref())",
        "proof_collection_active_id(",
        "proof_collection_assets_in_visible_order(",
        "ImUiMultiSelectState::new(duplicated_ids.clone(), Some(anchor))",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_duplicate_selection_source.contains(needle),
            "the demo-local collection duplicate selection owner should keep duplicate insertion and reselect repair explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_duplicate_shortcut_matches_primary_d_only",
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "pub(super) struct ProofCollectionDuplicateNameRegistry",
        "ProofCollectionDeleteResult",
        "proof_collection_delete_selection(",
        "render_collection_first_asset_browser_proof",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy",
        "proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists",
        "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
        "proof_collection_delete_selection_picks_previous_visible_item_at_end",
    ] {
        assert!(
            !selection_duplicate_selection_source.contains(needle),
            "the demo-local collection duplicate selection owner should not take shortcut, naming internals, delete, render, or UI policy; unexpected `{needle}`"
        );
    }

    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "authoring_parity_collection_assets()",
        "proof_collection_duplicate_selection_result(",
        "proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy",
        "proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists",
    ] {
        assert!(
            selection_duplicate_selection_tests_source.contains(needle),
            "the demo-local collection duplicate selection tests owner should keep fixture imports and behavior coverage explicit; missing `{needle}`"
        );
    }

    for needle in [
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "pub(super) fn proof_collection_duplicate_selection_result(",
        "let mut duplicates_by_source = HashMap::<Arc<str>, ProofCollectionAsset>::new();",
        "pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_duplicate_shortcut_matches_primary_d_only",
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "pub(super) struct ProofCollectionDuplicateNameRegistry",
        "ProofCollectionDeleteResult",
        "proof_collection_delete_selection(",
        "render_collection_first_asset_browser_proof",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
        "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
        "proof_collection_delete_selection_picks_previous_visible_item_at_end",
    ] {
        assert!(
            !selection_duplicate_selection_tests_source.contains(needle),
            "the demo-local collection duplicate selection tests owner should not take duplicate command flow, naming internals, delete, render, or UI policy; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn selection_state(",
        "pub(super) fn selected_ids(",
        "pub(super) fn anchor_id(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            selection_duplicate_selection_tests_fixtures_source.contains(needle),
            "the demo-local collection duplicate selection tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy",
        "proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists",
        "authoring_parity_collection_assets()",
        "proof_collection_duplicate_selection_result(",
        "pub(super) fn proof_collection_duplicate_selection_result(",
        "pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_duplicate_shortcut_matches_primary_d_only",
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "pub(super) struct ProofCollectionDuplicateNameRegistry",
        "ProofCollectionDeleteResult",
        "proof_collection_delete_selection(",
        "proof_collection_delete_key_matches(",
        "render_collection_first_asset_browser_proof",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !selection_duplicate_selection_tests_fixtures_source.contains(needle),
            "the demo-local collection duplicate selection tests fixture owner should not take behavior tests, duplicate command flow, naming internals, delete, render, or UI policy; unexpected `{needle}`"
        );
    }
}
