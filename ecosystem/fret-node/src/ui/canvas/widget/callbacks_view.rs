mod node_drag;
mod view_change;
mod viewport;

use super::*;

pub(super) fn emit_move_start<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
) {
    viewport::emit_move_start(canvas, snapshot, kind)
}

pub(super) fn emit_move_end<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
    outcome: ViewportMoveEndOutcome,
) {
    viewport::emit_move_end(canvas, snapshot, kind, outcome)
}

pub(super) fn emit_node_drag_start<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    primary: GraphNodeId,
    nodes: &[GraphNodeId],
) {
    node_drag::emit_node_drag_start(canvas, primary, nodes)
}

pub(super) fn emit_node_drag_end<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    primary: GraphNodeId,
    nodes: &[GraphNodeId],
    outcome: NodeDragEndOutcome,
) {
    node_drag::emit_node_drag_end(canvas, primary, nodes, outcome)
}

pub(super) fn emit_node_drag<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    primary: GraphNodeId,
    nodes: &[GraphNodeId],
) {
    node_drag::emit_node_drag(canvas, primary, nodes)
}

pub(super) fn emit_view_callbacks<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    changes: &[ViewChange],
) {
    view_change::emit_view_callbacks(canvas, changes)
}
