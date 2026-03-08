use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect, CanvasSize, NodeId as GraphNodeId};
use crate::io::NodeGraphNodeOrigin;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn clamp_anchor_in_rect_with_size(
    anchor: CanvasPoint,
    size: CanvasSize,
    extent: CanvasRect,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    super::node_drag_constraints_anchor::clamp_anchor_in_rect_with_size(
        anchor,
        size,
        extent,
        node_origin,
    )
}

pub(super) fn union_rect(a: CanvasRect, b: CanvasRect) -> CanvasRect {
    super::node_drag_constraints_anchor::union_rect(a, b)
}

pub(super) fn apply_multi_drag_extent_delta<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    node_ids: &[GraphNodeId],
    delta: CanvasPoint,
    multi_drag: bool,
) -> CanvasPoint {
    super::node_drag_constraints_extent::apply_multi_drag_extent_delta(
        canvas, cx, snapshot, node_ids, delta, multi_drag,
    )
}
