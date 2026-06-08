use super::*;

#[test]
fn proof_collection_drag_rect_normalizes_drag_direction() {
    let rect = proof_collection_drag_rect(
        Point::new(Px(48.0), Px(60.0)),
        Point::new(Px(12.0), Px(18.0)),
    );

    assert_eq!(rect.origin, Point::new(Px(12.0), Px(18.0)));
    assert_eq!(rect.size, Size::new(Px(36.0), Px(42.0)));
}

#[test]
fn proof_collection_layout_metrics_fall_back_before_viewport_binding_exists() {
    let layout =
        proof_collection_layout_metrics(Px(0.0), Px(PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX));

    assert_eq!(layout.columns, PROOF_COLLECTION_GRID_FALLBACK_COLUMNS);
    assert_eq!(
        layout.viewport_width,
        Px(PROOF_COLLECTION_GRID_FALLBACK_VIEWPORT_PX)
    );
    assert_eq!(
        layout.tile_extent,
        Px(PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX)
    );
    assert_eq!(
        layout.tile_min_height,
        Px(PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX + PROOF_COLLECTION_TILE_METADATA_PX)
    );
}
