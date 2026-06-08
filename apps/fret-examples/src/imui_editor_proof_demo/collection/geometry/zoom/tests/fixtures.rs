use fret_core::{Modifiers, Point, Px};

use super::super::{ProofCollectionLayoutMetrics, proof_collection_layout_metrics};

pub(super) fn zoom_layout() -> ProofCollectionLayoutMetrics {
    proof_collection_layout_metrics(Px(320.0), Px(96.0))
}

pub(super) fn scroll_offset_for_anchor_update() -> Point {
    Point::new(Px(0.0), Px(88.0))
}

pub(super) fn ignored_scroll_offset() -> Point {
    Point::new(Px(0.0), Px(24.0))
}

pub(super) fn updated_pointer_local() -> Point {
    Point::new(Px(140.0), Px(140.0))
}

pub(super) fn updated_wheel_delta() -> Point {
    Point::new(Px(0.0), Px(18.0))
}

pub(super) fn ignored_wheel_delta() -> Point {
    Point::new(Px(0.0), Px(12.0))
}

pub(super) fn primary_modifier() -> Modifiers {
    Modifiers {
        meta: true,
        ..Default::default()
    }
}

pub(super) fn asset_count() -> usize {
    6
}
