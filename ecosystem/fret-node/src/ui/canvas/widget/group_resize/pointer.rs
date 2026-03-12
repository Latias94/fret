use fret_core::{Point, Px, Rect};

use crate::core::CanvasPoint;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::*;

pub(super) fn auto_pan_delta<M: NodeGraphCanvasMiddleware>(
    snapshot: &ViewSnapshot,
    position: Point,
    bounds: Rect,
) -> CanvasPoint {
    snapshot
        .interaction
        .auto_pan
        .on_node_drag
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, bounds))
        .unwrap_or_default()
}

pub(super) fn adjusted_position(position: Point, auto_pan_delta: CanvasPoint) -> Point {
    Point::new(
        Px(position.x.0 - auto_pan_delta.x),
        Px(position.y.0 - auto_pan_delta.y),
    )
}
