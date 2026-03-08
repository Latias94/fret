use super::*;

pub(super) fn emit_connect_start<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: &WireDragKind,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    let event = ConnectStart {
        kind: drag_kind_from_wire_drag_kind(kind),
        mode: snapshot.interaction.connection_mode,
    };
    callbacks.on_connect_start(event.clone());
    if matches!(
        kind,
        WireDragKind::Reconnect { .. } | WireDragKind::ReconnectMany { .. }
    ) {
        callbacks.on_reconnect_start(event.clone());
        callbacks.on_edge_update_start(event);
    }
}

pub(super) fn emit_connect_end<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    mode: crate::interaction::NodeGraphConnectionMode,
    kind: &WireDragKind,
    target: Option<PortId>,
    outcome: ConnectEndOutcome,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    let event = ConnectEnd {
        kind: drag_kind_from_wire_drag_kind(kind),
        mode,
        target,
        outcome,
    };
    callbacks.on_connect_end(event.clone());
    if matches!(
        kind,
        WireDragKind::Reconnect { .. } | WireDragKind::ReconnectMany { .. }
    ) {
        callbacks.on_reconnect_end(event.clone());
        callbacks.on_edge_update_end(event);
    }
}

fn drag_kind_from_wire_drag_kind(kind: &WireDragKind) -> ConnectDragKind {
    match kind {
        WireDragKind::New { from, bundle } => ConnectDragKind::New {
            from: *from,
            bundle: bundle.clone(),
        },
        WireDragKind::Reconnect {
            edge,
            endpoint,
            fixed,
        } => ConnectDragKind::Reconnect {
            edge: *edge,
            endpoint: *endpoint,
            fixed: *fixed,
        },
        WireDragKind::ReconnectMany { edges } => ConnectDragKind::ReconnectMany {
            edges: edges.clone(),
        },
    }
}
