pub(super) fn assert_selection_context_menu_owner_split(
    selection_context_menu_source: &str,
    selection_context_menu_tests_source: &str,
    selection_context_menu_tests_fixtures_source: &str,
) {
    for needle in [
        "pub(in super::super) fn proof_collection_context_menu_selection(",
        "ImUiMultiSelectState::single(asset_id.clone())",
        "ProofCollectionKeyboardState {\n            active_id: Some(asset_id),",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_context_menu_source.contains(needle),
            "the demo-local collection context-menu selection owner should keep right-click selection policy explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "proof_collection_context_menu_selection(",
        "proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile",
        "proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile",
    ] {
        assert!(
            selection_context_menu_tests_source.contains(needle),
            "the demo-local collection context-menu selection tests owner should keep fixture imports and behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile",
        "proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile",
        "render_collection_first_asset_browser_proof",
        "proof_collection_keyboard_selection(",
        "proof_collection_select_all_selection(",
        "proof_collection_duplicate_selection(",
        "proof_collection_delete_selection(",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !selection_context_menu_source.contains(needle),
            "the demo-local collection context-menu selection owner should not take test fixtures, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "ImUiMultiSelectState::single(asset_id.clone())",
        "ProofCollectionKeyboardState {",
        "render_collection_first_asset_browser_proof",
        "proof_collection_keyboard_selection(",
        "proof_collection_select_all_selection(",
        "proof_collection_duplicate_selection(",
        "proof_collection_delete_selection(",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !selection_context_menu_tests_source.contains(needle),
            "the demo-local collection context-menu selection tests owner should not take policy construction, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selection_state(",
        "pub(super) fn selected_ids(",
        "pub(super) fn anchor_id(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            selection_context_menu_tests_fixtures_source.contains(needle),
            "the demo-local collection context-menu selection tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_context_menu_selection(",
        "proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile",
        "proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile",
        "ImUiMultiSelectState::single(asset_id.clone())",
        "ProofCollectionKeyboardState {",
        "render_collection_first_asset_browser_proof",
        "proof_collection_keyboard_selection(",
        "proof_collection_select_all_selection(",
        "proof_collection_duplicate_selection(",
        "proof_collection_delete_selection(",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !selection_context_menu_tests_fixtures_source.contains(needle),
            "the demo-local collection context-menu selection tests fixture owner should not take behavior tests, policy construction, render, command, or UI policy; unexpected `{needle}`"
        );
    }
}
