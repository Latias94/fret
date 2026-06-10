pub(super) fn assert_geometry_owner_split(
    geometry_source: &str,
    geometry_tests_source: &str,
    geometry_zoom_source: &str,
    geometry_zoom_tests_source: &str,
    geometry_zoom_tests_fixtures_source: &str,
) {
    for needle in [
        "mod zoom;",
        "pub(super) use zoom::{",
        "ProofCollectionZoomUpdate, proof_collection_zoom_line, proof_collection_zoom_request,",
        "#[cfg(test)]",
        "mod tests;",
        "pub(super) struct ProofCollectionLayoutMetrics",
        "pub(super) fn proof_collection_localize_rect(",
    ] {
        assert!(
            geometry_source.contains(needle),
            "the demo-local collection geometry owner should keep base layout/drag geometry and zoom re-exports explicit; missing `{needle}`"
        );
    }
    for needle in [
        "struct ProofCollectionZoomUpdate {",
        "fn proof_collection_zoom_line(",
        "fn proof_collection_zoom_modifier_active(",
        "fn proof_collection_hovered_index(",
        "fn proof_collection_zoom_request(",
        "fn proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor() {",
        "fn proof_collection_zoom_request_ignores_non_primary_wheel() {",
        "fn proof_collection_drag_rect_normalizes_drag_direction() {",
        "fn proof_collection_layout_metrics_fall_back_before_viewport_binding_exists() {",
    ] {
        assert!(
            !geometry_source.contains(needle),
            "the demo-local collection geometry owner should route split behavior coverage through child test owners; unexpected `{needle}`"
        );
    }

    for needle in [
        "proof_collection_drag_rect(",
        "proof_collection_layout_metrics(",
        "fn proof_collection_drag_rect_normalizes_drag_direction() {",
        "fn proof_collection_layout_metrics_fall_back_before_viewport_binding_exists() {",
    ] {
        assert!(
            geometry_tests_source.contains(needle),
            "the demo-local collection geometry tests owner should keep base geometry behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_localize_rect(",
        "pub(super) fn proof_collection_drag_rect(",
        "pub(super) fn proof_collection_rects_intersect(",
        "pub(super) fn proof_collection_layout_metrics(",
        "const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX",
        "pub(in super::super) struct ProofCollectionZoomUpdate",
        "pub(in super::super) fn proof_collection_zoom_line(",
        "pub(in super::super) fn proof_collection_zoom_request(",
    ] {
        assert!(
            !geometry_tests_source.contains(needle),
            "the demo-local collection geometry tests owner should not take implementation ownership; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionZoomUpdate",
        "pub(in super::super) fn proof_collection_zoom_line(",
        "fn proof_collection_zoom_modifier_active(",
        "fn proof_collection_hovered_index(",
        "pub(in super::super) fn proof_collection_zoom_request(",
        "proof_collection_clamp_tile_extent(",
        "proof_collection_layout_metrics(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            geometry_zoom_source.contains(needle),
            "the demo-local collection geometry zoom owner should keep zoom math and tests routing explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "proof_collection_zoom_request(",
        "fn proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor() {",
        "fn proof_collection_zoom_request_ignores_non_primary_wheel() {",
    ] {
        assert!(
            geometry_zoom_tests_source.contains(needle),
            "the demo-local collection geometry zoom tests owner should keep zoom behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_localize_rect(",
        "pub(super) fn proof_collection_drag_rect(",
        "pub(super) fn proof_collection_rects_intersect(",
        "pub(super) fn proof_collection_layout_metrics(",
        "const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX",
        "fn proof_collection_drag_rect_normalizes_drag_direction() {",
        "fn proof_collection_layout_metrics_fall_back_before_viewport_binding_exists() {",
        "fn proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor() {",
        "fn proof_collection_zoom_request_ignores_non_primary_wheel() {",
    ] {
        assert!(
            !geometry_zoom_source.contains(needle),
            "the demo-local collection geometry zoom owner should not take base layout/drag geometry; unexpected `{needle}`"
        );
    }
    for needle in [
        "Point::new(Px(0.0), Px(88.0))",
        "Point::new(Px(140.0), Px(140.0))",
        "Point::new(Px(0.0), Px(18.0))",
        "Modifiers {",
        "pub(in super::super) struct ProofCollectionZoomUpdate",
        "pub(in super::super) fn proof_collection_zoom_line(",
        "fn proof_collection_zoom_modifier_active(",
        "fn proof_collection_hovered_index(",
        "pub(in super::super) fn proof_collection_zoom_request(",
        "proof_collection_clamp_tile_extent(",
        "pub(super) fn proof_collection_localize_rect(",
        "pub(super) fn proof_collection_drag_rect(",
        "pub(super) fn proof_collection_rects_intersect(",
        "const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX",
    ] {
        assert!(
            !geometry_zoom_tests_source.contains(needle),
            "the demo-local collection geometry zoom tests owner should not take zoom implementation or base geometry; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn zoom_layout() -> ProofCollectionLayoutMetrics",
        "proof_collection_layout_metrics(Px(320.0), Px(96.0))",
        "pub(super) fn primary_modifier() -> Modifiers",
        "meta: true",
        "pub(super) fn asset_count() -> usize",
    ] {
        assert!(
            geometry_zoom_tests_fixtures_source.contains(needle),
            "the demo-local collection geometry zoom tests fixture owner should keep zoom request setup explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor() {",
        "fn proof_collection_zoom_request_ignores_non_primary_wheel() {",
        "proof_collection_zoom_request(",
        "pub(in super::super) struct ProofCollectionZoomUpdate",
        "pub(in super::super) fn proof_collection_zoom_line(",
        "fn proof_collection_zoom_modifier_active(",
        "fn proof_collection_hovered_index(",
        "pub(in super::super) fn proof_collection_zoom_request(",
        "proof_collection_clamp_tile_extent(",
        "pub(super) fn proof_collection_localize_rect(",
        "pub(super) fn proof_collection_drag_rect(",
        "pub(super) fn proof_collection_rects_intersect(",
        "const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX",
        "render_collection_first_asset_browser_proof",
        "TextField",
        "PointerRegionProps",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !geometry_zoom_tests_fixtures_source.contains(needle),
            "the demo-local collection geometry zoom tests fixture owner should not take behavior tests, zoom implementation, render, or UI policy; unexpected `{needle}`"
        );
    }
}
