use super::*;

pub(super) fn emit_move_start<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_move_start(ViewportMoveStart {
        kind,
        pan: snapshot.pan,
        zoom: snapshot.zoom,
    });
}

pub(super) fn emit_move_end<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
    outcome: ViewportMoveEndOutcome,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_move_end(ViewportMoveEnd {
        kind,
        pan: snapshot.pan,
        zoom: snapshot.zoom,
        outcome,
    });
}

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

pub(super) fn emit_view_callbacks<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    changes: &[ViewChange],
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    if changes.is_empty() {
        return;
    }

    callbacks.on_view_change(changes);
    for change in changes {
        match change {
            ViewChange::Viewport { pan, zoom } => {
                callbacks.on_viewport_change(*pan, *zoom);
                callbacks.on_move(*pan, *zoom);
            }
            ViewChange::Selection {
                nodes,
                edges,
                groups,
            } => callbacks.on_selection_change(crate::runtime::callbacks::SelectionChange {
                nodes: nodes.clone(),
                edges: edges.clone(),
                groups: groups.clone(),
            }),
        }
    }
}
