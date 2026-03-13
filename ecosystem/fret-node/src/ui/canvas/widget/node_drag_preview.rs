mod compute;
mod state;

use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect, GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::state::NodeDrag;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn compute_preview_positions<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    delta: CanvasPoint,
    multi_drag: bool,
) -> (Vec<(GraphNodeId, CanvasPoint)>, Vec<(GroupId, CanvasRect)>) {
    compute::compute_preview_positions(canvas, cx, snapshot, drag, delta, multi_drag)
}

pub(super) fn update_drag_preview_state(
    drag: &mut NodeDrag,
    next_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    next_groups: Vec<(GroupId, CanvasRect)>,
) {
    state::update_drag_preview_state(drag, next_nodes, next_groups)
}
