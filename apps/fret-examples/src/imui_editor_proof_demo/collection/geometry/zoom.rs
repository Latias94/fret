use fret_core::{Modifiers, Point, Px};

use super::{
    PROOF_COLLECTION_TILE_EXTENT_STEP_PX, ProofCollectionLayoutMetrics,
    proof_collection_clamp_tile_extent, proof_collection_layout_metrics,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in super::super) struct ProofCollectionZoomUpdate {
    pub(in super::super) next_tile_extent: Px,
    pub(in super::super) next_scroll_offset: Point,
}

pub(in super::super) fn proof_collection_zoom_line(layout: ProofCollectionLayoutMetrics) -> String {
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

pub(in super::super) fn proof_collection_zoom_request(
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
