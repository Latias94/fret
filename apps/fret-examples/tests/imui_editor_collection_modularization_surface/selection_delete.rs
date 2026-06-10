pub(super) fn assert_selection_delete_owner_split(
    selection_delete_commands_source: &str,
    selection_delete_commands_tests_source: &str,
    selection_delete_commands_tests_fixtures_source: &str,
) {
    for needle in [
        "pub(in super::super::super) struct ProofCollectionDeleteResult",
        "pub(in super::super::super) fn proof_collection_delete_selection(",
        "pub(in super::super::super) fn proof_collection_delete_key_matches(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_delete_commands_source.contains(needle),
            "the demo-local collection delete command owner should keep delete/refocus transitions explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "authoring_parity_collection_assets()",
        "proof_collection_assets_in_visible_order(",
        "proof_collection_delete_selection(",
        "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
        "proof_collection_delete_selection_picks_previous_visible_item_at_end",
    ] {
        assert!(
            selection_delete_commands_tests_source.contains(needle),
            "the demo-local collection delete command tests owner should keep fixture imports and delete/refocus behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "ProofCollectionDuplicateResult",
        "proof_collection_duplicate_selection(",
        "proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "ProofCollectionDuplicateNameRegistry",
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
            !selection_delete_commands_tests_source.contains(needle),
            "the demo-local collection delete command tests owner should not take fixtures, duplicate, render, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selection_state(",
        "pub(super) fn selected_ids(",
        "pub(super) fn anchor_id(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            selection_delete_commands_tests_fixtures_source.contains(needle),
            "the demo-local collection delete command tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
        "proof_collection_delete_selection_picks_previous_visible_item_at_end",
        "authoring_parity_collection_assets()",
        "proof_collection_assets_in_visible_order(",
        "proof_collection_delete_selection(",
        "ProofCollectionDuplicateResult",
        "proof_collection_duplicate_selection(",
        "proof_collection_duplicate_shortcut_matches(",
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "ProofCollectionDuplicateNameRegistry",
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
            !selection_delete_commands_tests_fixtures_source.contains(needle),
            "the demo-local collection delete command tests fixture owner should not take behavior tests, delete command flow, duplicate, render, or UI policy; unexpected `{needle}`"
        );
    }
}
