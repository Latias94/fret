use super::super::*;

pub(super) fn emit_node_drag_start<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    primary: GraphNodeId,
    nodes: &[GraphNodeId],
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_node_drag_start(NodeDragStart {
        primary,
        nodes: nodes.to_vec(),
    });
}

pub(super) fn emit_node_drag_end<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    primary: GraphNodeId,
    nodes: &[GraphNodeId],
    outcome: NodeDragEndOutcome,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_node_drag_end(NodeDragEnd {
        primary,
        nodes: nodes.to_vec(),
        outcome,
    });
}

pub(super) fn emit_node_drag<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    primary: GraphNodeId,
    nodes: &[GraphNodeId],
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_node_drag(primary, nodes);
}
