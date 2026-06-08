use super::*;

#[test]
fn proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor() {
    let layout = proof_collection_layout_metrics(Px(320.0), Px(96.0));

    let update = proof_collection_zoom_request(
        layout,
        Point::new(Px(0.0), Px(88.0)),
        Point::new(Px(140.0), Px(140.0)),
        Point::new(Px(0.0), Px(18.0)),
        Modifiers {
            meta: true,
            ..Default::default()
        },
        6,
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
    let layout = proof_collection_layout_metrics(Px(320.0), Px(96.0));

    assert!(
        proof_collection_zoom_request(
            layout,
            Point::new(Px(0.0), Px(24.0)),
            Point::new(Px(80.0), Px(48.0)),
            Point::new(Px(0.0), Px(12.0)),
            Modifiers::default(),
            6,
        )
        .is_none(),
        "collection zoom should stay opt-in on primary+wheel so plain wheel can keep scrolling"
    );
}
