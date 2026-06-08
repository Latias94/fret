use fret_core::{Point, Px, Rect, Size};

mod zoom;

pub(super) use zoom::{
    ProofCollectionZoomUpdate, proof_collection_zoom_line, proof_collection_zoom_request,
};

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

#[cfg(test)]
mod tests;
