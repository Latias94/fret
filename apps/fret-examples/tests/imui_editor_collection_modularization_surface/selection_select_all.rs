pub(super) fn assert_selection_select_all_owner_split(
    selection_source: &str,
    selection_select_all_source: &str,
    selection_select_all_tests_source: &str,
    selection_select_all_tests_fixtures_source: &str,
) {
    for needle in [
        "pub(in super::super) fn proof_collection_select_all_shortcut_matches(",
        "pub(in super::super) fn proof_collection_select_all_selection(",
        "proof_collection_active_id(collection_keys, selection, keyboard)",
        "ImUiMultiSelectState::from_ordered_selection(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_select_all_source.contains(needle),
            "the demo-local collection select-all owner should keep shortcut and full visible-order selection policy explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "ProofCollectionKeyboardState",
        "proof_collection_select_all_selection(",
        "proof_collection_select_all_shortcut_matches(",
        "proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile",
        "proof_collection_select_all_selection_falls_back_to_first_visible_asset",
        "proof_collection_select_all_shortcut_matches_primary_a_only",
    ] {
        assert!(
            selection_select_all_tests_source.contains(needle),
            "the demo-local collection select-all tests owner should keep fixture imports and behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn anchor_id(",
        "ImUiMultiSelectState::new(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "render_collection_first_asset_browser_proof",
        "proof_collection_keyboard_selection(",
        "proof_collection_context_menu_selection(",
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
            !selection_select_all_tests_source.contains(needle),
            "the demo-local collection select-all tests owner should not take selection fixture construction, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selection_state(",
        "pub(super) fn anchor_id(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            selection_select_all_tests_fixtures_source.contains(needle),
            "the demo-local collection select-all tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn anchor_id(",
        "proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile",
        "proof_collection_select_all_selection_falls_back_to_first_visible_asset",
        "proof_collection_select_all_shortcut_matches_primary_a_only",
        "render_collection_first_asset_browser_proof",
        "proof_collection_keyboard_selection(",
        "proof_collection_context_menu_selection(",
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
            !selection_select_all_source.contains(needle),
            "the demo-local collection select-all owner should not take test fixtures, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "ImUiMultiSelectState::from_ordered_selection(",
        "render_collection_first_asset_browser_proof",
        "proof_collection_keyboard_selection(",
        "proof_collection_context_menu_selection(",
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
            !selection_select_all_tests_fixtures_source.contains(needle),
            "the demo-local collection select-all tests fixture owner should not take behavior tests, policy construction, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_context_menu_selection(",
        "proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile",
        "proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile",
        "pub(super) fn proof_collection_keyboard_selection(",
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
        "pub(super) fn proof_collection_assets_in_visible_order(",
        "pub(super) fn proof_collection_selected_assets",
        "pub(super) fn proof_collection_active_id(",
        "collect::<HashMap<_, _>>()",
        "pub(super) fn proof_collection_select_all_shortcut_matches(",
        "pub(super) fn proof_collection_select_all_selection(",
        "proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile",
        "proof_collection_select_all_selection_falls_back_to_first_visible_asset",
        "proof_collection_select_all_shortcut_matches_primary_a_only",
    ] {
        assert!(
            !selection_source.contains(needle),
            "the demo-local collection selection root should delegate select-all policy to selection/select_all.rs; unexpected `{needle}`"
        );
    }
}
