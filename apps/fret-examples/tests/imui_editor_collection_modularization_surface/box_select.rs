pub(super) fn assert_box_select_owner_split(
    box_select_source: &str,
    box_select_tests_source: &str,
    box_select_tests_fixtures_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionBoxSelectSession",
        "pub(super) struct ProofCollectionBoxSelectState",
        "pub(super) struct ProofCollectionRenderedItem",
        "pub(super) fn proof_collection_box_select_selection(",
        "pub(super) fn proof_collection_box_select_active_rect(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            box_select_source.contains(needle),
            "the demo-local collection box-select owner should keep marquee selection state explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_box_select_state_for_hits(",
        "proof_collection_box_select_selection(",
        "mod fixtures;",
        "use fixtures::{",
        "fn proof_collection_box_select_replace_uses_visible_collection_order() {",
        "fn proof_collection_box_select_append_preserves_baseline_and_adds_hits() {",
    ] {
        assert!(
            box_select_tests_source.contains(needle),
            "the demo-local collection box-select tests owner should keep behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn selected_ids(",
        "fn anchor_id(",
        "fn proof_collection_box_select_replace_uses_visible_collection_order() {",
        "fn proof_collection_box_select_append_preserves_baseline_and_adds_hits() {",
    ] {
        assert!(
            !box_select_source.contains(needle),
            "the demo-local collection box-select owner should route behavior coverage through box_select/tests.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "fn selected_ids(",
        "fn anchor_id(",
        "pub(super) struct ProofCollectionRenderedItem",
        "pub(super) struct ProofCollectionBoxSelectSession",
        "pub(super) struct ProofCollectionBoxSelectState",
        "fn proof_collection_box_select_hits(",
        "pub(super) fn proof_collection_box_select_active_rect(",
    ] {
        assert!(
            !box_select_tests_source.contains(needle),
            "the demo-local collection box-select tests owner should not take implementation ownership; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selected_ids(",
        "pub(super) fn anchor_id(",
        "ImUiMultiSelectState",
    ] {
        assert!(
            box_select_tests_fixtures_source.contains(needle),
            "the demo-local collection box-select tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_box_select_replace_uses_visible_collection_order",
        "proof_collection_box_select_append_preserves_baseline_and_adds_hits",
        "proof_collection_box_select_state_for_hits(",
        "proof_collection_box_select_selection(",
        "pub(super) struct ProofCollectionRenderedItem",
        "pub(super) struct ProofCollectionBoxSelectSession",
        "pub(super) struct ProofCollectionBoxSelectState",
        "fn proof_collection_box_select_hits(",
        "pub(super) fn proof_collection_box_select_active_rect(",
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
            !box_select_tests_fixtures_source.contains(needle),
            "the demo-local collection box-select tests fixture owner should not take behavior tests, box-select implementation, render, or UI policy; unexpected `{needle}`"
        );
    }
}
