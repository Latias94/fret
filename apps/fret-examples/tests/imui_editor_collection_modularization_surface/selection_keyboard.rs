pub(super) fn assert_selection_keyboard_owner_split(
    selection_keyboard_source: &str,
    selection_keyboard_tests_source: &str,
    selection_keyboard_tests_fixtures_source: &str,
    selection_keyboard_navigation_source: &str,
    selection_keyboard_navigation_tests_source: &str,
    selection_keyboard_navigation_tests_fixtures_source: &str,
) {
    for needle in [
        "mod navigation;",
        "use navigation::{",
        "pub(in super::super) fn proof_collection_keyboard_selection(",
        "proof_collection_active_id(collection_keys, selection, keyboard)",
        "proof_collection_keyboard_next_index(",
        "proof_collection_keyboard_move_selection(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_keyboard_source.contains(needle),
            "the demo-local collection keyboard selection owner should keep arrow/range/Escape policy explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "authoring_parity_collection_assets()",
        "PROOF_COLLECTION_GRID_FALLBACK_COLUMNS",
        "proof_collection_keyboard_selection(",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
    ] {
        assert!(
            selection_keyboard_tests_source.contains(needle),
            "the demo-local collection keyboard selection tests owner should keep fixture imports and behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_keyboard_navigation_source.contains(needle),
            "the demo-local collection keyboard navigation owner should keep next-index and range selection construction explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_keyboard_next_index(",
        "proof_collection_keyboard_move_selection(",
        "mod fixtures;",
        "use fixtures::{",
        "fn proof_collection_keyboard_next_index_moves_with_columns_and_edges() {",
        "fn proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order() {",
    ] {
        assert!(
            selection_keyboard_navigation_tests_source.contains(needle),
            "the demo-local collection keyboard navigation tests owner should keep fixture imports plus next-index and range selection coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "proof_collection_keyboard_next_index_moves_with_columns_and_edges",
        "proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order",
        "fn selection_state(",
        "fn selected_ids(",
        "fn anchor_id(",
        "authoring_parity_collection_assets()",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
    ] {
        assert!(
            !selection_keyboard_source.contains(needle),
            "the demo-local collection keyboard selection owner should route navigation helpers through keyboard/navigation.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(in super::super) fn proof_collection_keyboard_selection(",
        "proof_collection_active_id(",
        "KeyCode::Escape",
        "modifiers.alt",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
        "fn keys() -> Vec<Arc<str>>",
        "fn selection_state(",
        "fn selected_ids(",
        "fn proof_collection_keyboard_next_index_moves_with_columns_and_edges() {",
        "fn proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order() {",
    ] {
        assert!(
            !selection_keyboard_navigation_source.contains(needle),
            "the demo-local collection keyboard navigation owner should not take keyboard policy entry, active-id fallback, or modifier filtering; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "pub(in super::super) fn proof_collection_keyboard_selection(",
        "proof_collection_active_id(",
        "KeyCode::Escape",
        "modifiers.alt",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
        "fn keys() -> Vec<Arc<str>>",
        "fn selection_state(",
        "fn selected_ids(",
        "render_collection_first_asset_browser_proof",
        "proof_collection_select_all_selection(",
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
            !selection_keyboard_navigation_tests_source.contains(needle),
            "the demo-local collection keyboard navigation tests owner should not take navigation implementation, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn keys() -> Vec<Arc<str>>",
        "pub(super) fn selection_state(",
        "pub(super) fn selected_ids(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            selection_keyboard_navigation_tests_fixtures_source.contains(needle),
            "the demo-local collection keyboard navigation tests fixture owner should keep navigation fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_keyboard_next_index(",
        "proof_collection_keyboard_move_selection(",
        "proof_collection_keyboard_next_index_moves_with_columns_and_edges",
        "proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order",
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "pub(in super::super) fn proof_collection_keyboard_selection(",
        "proof_collection_active_id(",
        "KeyCode::Escape",
        "modifiers.alt",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
        "render_collection_first_asset_browser_proof",
        "proof_collection_select_all_selection(",
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
            !selection_keyboard_navigation_tests_fixtures_source.contains(needle),
            "the demo-local collection keyboard navigation tests fixture owner should not take behavior tests, navigation implementation, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "proof_collection_keyboard_next_index_moves_with_columns_and_edges",
        "proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order",
        "render_collection_first_asset_browser_proof",
        "proof_collection_select_all_selection(",
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
            !selection_keyboard_tests_source.contains(needle),
            "the demo-local collection keyboard selection tests owner should not take navigation helpers, render, command, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selection_state(",
        "pub(super) fn selected_ids(",
        "pub(super) fn anchor_id(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            selection_keyboard_tests_fixtures_source.contains(needle),
            "the demo-local collection keyboard selection tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "authoring_parity_collection_assets()",
        "PROOF_COLLECTION_GRID_FALLBACK_COLUMNS",
        "proof_collection_keyboard_selection(",
        "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
        "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
        "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
        "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
        "pub(super) fn proof_collection_keyboard_next_index(",
        "pub(super) fn proof_collection_keyboard_move_selection(",
        "ImUiMultiSelectState::from_ordered_selection(",
        "proof_collection_keyboard_next_index_moves_with_columns_and_edges",
        "proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order",
        "render_collection_first_asset_browser_proof",
        "proof_collection_select_all_selection(",
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
            !selection_keyboard_tests_fixtures_source.contains(needle),
            "the demo-local collection keyboard selection tests fixture owner should not take behavior tests, navigation helpers, render, command, or UI policy; unexpected `{needle}`"
        );
    }
}
