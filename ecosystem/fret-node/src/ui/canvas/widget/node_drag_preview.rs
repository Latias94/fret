mod compute;
mod state;

use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect, GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::state::NodeDrag;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    node_drag_preview_cx::NodeDragPreviewCx,
};

pub(super) fn compute_preview_positions<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    delta: CanvasPoint,
    multi_drag: bool,
) -> (Vec<(GraphNodeId, CanvasPoint)>, Vec<(GroupId, CanvasRect)>)
where
    M: NodeGraphCanvasMiddleware,
    Cx: NodeDragPreviewCx<H>,
{
    compute::compute_preview_positions(canvas, cx, snapshot, drag, delta, multi_drag)
}

pub(super) fn update_drag_preview_state(
    drag: &mut NodeDrag,
    next_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    next_groups: Vec<(GroupId, CanvasRect)>,
) {
    state::update_drag_preview_state(drag, next_nodes, next_groups)
}
