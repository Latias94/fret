use super::super::prelude::*;

pub(super) fn from_port_and_require_from_connectable_start(
    kind: &WireDragKind,
) -> (Option<PortId>, bool) {
    match kind {
        WireDragKind::New { from, .. } => (Some(*from), true),
        WireDragKind::Reconnect { fixed, .. } => (Some(*fixed), false),
        WireDragKind::ReconnectMany { edges } => (edges.first().map(|e| e.2), false),
    }
}
