use fret_core::{Modifiers, Point, Px, Rect, Size};

pub(super) const PROOF_COLLECTION_GRID_FALLBACK_COLUMNS: usize = 3;
pub(super) const PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX: f32 = 96.0;

const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX: f32 = 6.0;
const PROOF_COLLECTION_GRID_FALLBACK_VIEWPORT_PX: f32 = 320.0;
const PROOF_COLLECTION_TILE_EXTENT_MIN_PX: f32 = 72.0;
const PROOF_COLLECTION_TILE_EXTENT_MAX_PX: f32 = 160.0;
const PROOF_COLLECTION_TILE_EXTENT_STEP_PX: f32 = 16.0;
const PROOF_COLLECTION_TILE_METADATA_PX: f32 = 44.0;
const PROOF_COLLECTION_TILE_ROW_GAP_PX: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProofCollectionLayoutMetrics {
    pub(super) viewport_width: Px,
    pub(super) columns: usize,
    pub(super) tile_extent: Px,
    pub(super) tile_min_height: Px,
    pub(super) row_step: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProofCollectionZoomUpdate {
    pub(super) next_tile_extent: Px,
    pub(super) next_scroll_offset: Point,
}

fn proof_collection_point_sub(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 - b.x.0), Px(a.y.0 - b.y.0))
}

pub(super) fn proof_collection_localize_rect(rect: Rect, origin: Point) -> Rect {
    Rect::new(proof_collection_point_sub(rect.origin, origin), rect.size)
}

pub(super) fn proof_collection_drag_rect(origin_local: Point, current_local: Point) -> Rect {
    let left = origin_local.x.0.min(current_local.x.0);
    let top = origin_local.y.0.min(current_local.y.0);
    let right = origin_local.x.0.max(current_local.x.0);
    let bottom = origin_local.y.0.max(current_local.y.0);

    Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px(right - left), Px(bottom - top)),
    )
}

pub(super) fn proof_collection_drag_threshold_met(
    origin_local: Point,
    current_local: Point,
) -> bool {
    let dx = current_local.x.0 - origin_local.x.0;
    let dy = current_local.y.0 - origin_local.y.0;
    let distance_squared = dx * dx + dy * dy;
    distance_squared >= PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX.powi(2)
}

pub(super) fn proof_collection_rects_intersect(a: Rect, b: Rect) -> bool {
    let ax1 = a.origin.x.0 + a.size.width.0;
    let ay1 = a.origin.y.0 + a.size.height.0;
    let bx1 = b.origin.x.0 + b.size.width.0;
    let by1 = b.origin.y.0 + b.size.height.0;

    a.origin.x.0 < bx1 && ax1 > b.origin.x.0 && a.origin.y.0 < by1 && ay1 > b.origin.y.0
}

fn proof_collection_clamp_tile_extent(tile_extent: Px) -> Px {
    Px(tile_extent.0.clamp(
        PROOF_COLLECTION_TILE_EXTENT_MIN_PX,
        PROOF_COLLECTION_TILE_EXTENT_MAX_PX,
    ))
}

pub(super) fn proof_collection_layout_metrics(
    viewport_width: Px,
    tile_extent: Px,
) -> ProofCollectionLayoutMetrics {
    let tile_extent = proof_collection_clamp_tile_extent(tile_extent);
    let (viewport_width, columns) = if viewport_width.0 > 1.0 {
        (
            viewport_width,
            ((viewport_width.0 / tile_extent.0).floor() as usize).max(1),
        )
    } else {
        (
            Px(PROOF_COLLECTION_GRID_FALLBACK_VIEWPORT_PX),
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
        )
    };
    let tile_min_height = Px(tile_extent.0 + PROOF_COLLECTION_TILE_METADATA_PX);

    ProofCollectionLayoutMetrics {
        viewport_width,
        columns,
        tile_extent,
        tile_min_height,
        row_step: Px(tile_min_height.0 + PROOF_COLLECTION_TILE_ROW_GAP_PX),
    }
}

pub(super) fn proof_collection_zoom_line(layout: ProofCollectionLayoutMetrics) -> String {
    format!(
        "Primary+Wheel zoom stays app-owned: {} px target tiles across {} column(s), with hovered rows staying anchored inside the collection proof.",
        layout.tile_extent.0.round() as i32,
        layout.columns,
    )
}

fn proof_collection_zoom_modifier_active(modifiers: Modifiers) -> bool {
    !modifiers.alt && !modifiers.shift && (modifiers.ctrl || modifiers.meta)
}

fn proof_collection_hovered_index(
    layout: ProofCollectionLayoutMetrics,
    scroll_offset: Point,
    pointer_local: Point,
    asset_count: usize,
) -> Option<usize> {
    if asset_count == 0 {
        return None;
    }

    let row =
        (((pointer_local.y.0 + scroll_offset.y.0) / layout.row_step.0).floor()).max(0.0) as usize;
    let column_width = (layout.viewport_width.0 / layout.columns as f32).max(1.0);
    let col = ((pointer_local.x.0 / column_width).floor())
        .clamp(0.0, (layout.columns.saturating_sub(1)) as f32) as usize;

    Some((row * layout.columns + col).min(asset_count.saturating_sub(1)))
}

pub(super) fn proof_collection_zoom_request(
    layout: ProofCollectionLayoutMetrics,
    scroll_offset: Point,
    pointer_local: Point,
    wheel_delta: Point,
    modifiers: Modifiers,
    asset_count: usize,
) -> Option<ProofCollectionZoomUpdate> {
    if !proof_collection_zoom_modifier_active(modifiers) || wheel_delta.y.0.abs() <= 0.01 {
        return None;
    }

    let direction = if wheel_delta.y.0 > 0.0 { 1.0 } else { -1.0 };
    let next_tile_extent = proof_collection_clamp_tile_extent(Px(
        layout.tile_extent.0 + direction * PROOF_COLLECTION_TILE_EXTENT_STEP_PX
    ));
    if (next_tile_extent.0 - layout.tile_extent.0).abs() <= 0.01 {
        return None;
    }

    let next_layout = proof_collection_layout_metrics(layout.viewport_width, next_tile_extent);
    let next_scroll_offset = if let Some(index) =
        proof_collection_hovered_index(layout, scroll_offset, pointer_local, asset_count)
    {
        let current_row = index / layout.columns;
        let row_offset =
            (pointer_local.y.0 + scroll_offset.y.0) - current_row as f32 * layout.row_step.0;
        let next_row = index / next_layout.columns;
        Point::new(
            scroll_offset.x,
            Px(
                (next_row as f32 * next_layout.row_step.0 + row_offset - pointer_local.y.0)
                    .max(0.0),
            ),
        )
    } else {
        scroll_offset
    };

    Some(ProofCollectionZoomUpdate {
        next_tile_extent,
        next_scroll_offset,
    })
}

#[cfg(test)]
mod tests {
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
}
