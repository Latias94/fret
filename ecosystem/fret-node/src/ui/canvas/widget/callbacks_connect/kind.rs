use super::super::*;

pub(super) fn drag_kind_from_wire_drag_kind(kind: &WireDragKind) -> ConnectDragKind {
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
