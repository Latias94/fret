use super::*;

mod fixtures;

use fixtures::{
    asset_count, ignored_scroll_offset, ignored_wheel_delta, primary_modifier,
    scroll_offset_for_anchor_update, updated_pointer_local, updated_wheel_delta, zoom_layout,
};

#[test]
fn proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor() {
    let layout = zoom_layout();

    let update = proof_collection_zoom_request(
        layout,
        scroll_offset_for_anchor_update(),
        updated_pointer_local(),
        updated_wheel_delta(),
        primary_modifier(),
        asset_count(),
    )
    .expect("primary+wheel should produce a zoom request");

    assert_eq!(update.next_tile_extent, Px(112.0));
    assert_eq!(update.next_scroll_offset, Point::new(Px(0.0), Px(268.0)));
    assert_eq!(
        proof_collection_layout_metrics(layout.viewport_width, update.next_tile_extent).columns,
        2
    );
}

#[test]
fn proof_collection_zoom_request_ignores_non_primary_wheel() {
    let layout = zoom_layout();

    assert!(
        proof_collection_zoom_request(
            layout,
            ignored_scroll_offset(),
            updated_pointer_local(),
            ignored_wheel_delta(),
            Modifiers::default(),
            asset_count(),
        )
        .is_none(),
        "collection zoom should stay opt-in on primary+wheel so plain wheel can keep scrolling"
    );
}
