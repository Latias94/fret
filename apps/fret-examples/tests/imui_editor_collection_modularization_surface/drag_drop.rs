pub(super) fn assert_drag_drop_owner_split(
    drag_drop_source: &str,
    drag_drop_tests_source: &str,
    drag_drop_tests_fixtures_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionDragPayload",
        "pub(super) fn proof_collection_drag_payload_for_asset(",
        "pub(super) fn proof_collection_drag_preview_title(",
        "pub(super) fn proof_collection_drag_preview_subtitle(",
        "pub(super) fn proof_collection_drop_status(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            drag_drop_source.contains(needle),
            "the demo-local collection drag/drop owner should keep payload and status projection explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::selection_state;",
        "fn proof_collection_drag_payload_for_selected_asset_carries_selected_set() {",
        "fn proof_collection_drag_payload_for_unselected_asset_carries_dragged_asset_only() {",
        "proof_collection_drag_payload_for_asset(",
        "proof_collection_drag_preview_title(",
        "proof_collection_drag_preview_subtitle(",
        "proof_collection_drop_status(",
    ] {
        assert!(
            drag_drop_tests_source.contains(needle),
            "the demo-local collection drag/drop tests owner should keep drag payload behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "fn proof_collection_drag_payload_for_selected_asset_carries_selected_set() {",
        "fn proof_collection_drag_payload_for_unselected_asset_carries_dragged_asset_only() {",
    ] {
        assert!(
            !drag_drop_source.contains(needle),
            "the demo-local collection drag/drop owner should route behavior coverage through drag_drop/tests.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(",
        "pub(super) struct ProofCollectionDragPayload",
        "pub(super) fn proof_collection_drag_payload_for_asset(",
        "pub(super) fn proof_collection_drag_preview_title(",
        "pub(super) fn proof_collection_drag_preview_subtitle(",
        "pub(super) fn proof_collection_drop_status(",
    ] {
        assert!(
            !drag_drop_tests_source.contains(needle),
            "the demo-local collection drag/drop tests owner should not take implementation ownership; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selection_state(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            drag_drop_tests_fixtures_source.contains(needle),
            "the demo-local collection drag/drop tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_drag_payload_for_selected_asset_carries_selected_set",
        "proof_collection_drag_payload_for_unselected_asset_carries_dragged_asset_only",
        "proof_collection_drag_payload_for_asset(",
        "proof_collection_drag_preview_title(",
        "proof_collection_drag_preview_subtitle(",
        "proof_collection_drop_status(",
        "pub(super) struct ProofCollectionDragPayload",
        "pub(super) fn proof_collection_drag_payload_for_asset(",
        "pub(super) fn proof_collection_drag_preview_title(",
        "pub(super) fn proof_collection_drag_preview_subtitle(",
        "pub(super) fn proof_collection_drop_status(",
        "render_collection_first_asset_browser_proof",
        "drag_source_with_options",
        "drop_target::<",
        "drag_preview_ghost",
        "proof_drag_preview_card",
        "TextField",
        "PointerRegionProps",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !drag_drop_tests_fixtures_source.contains(needle),
            "the demo-local collection drag/drop tests fixture owner should not take behavior tests, drag/drop implementation, render, or UI policy; unexpected `{needle}`"
        );
    }
}
