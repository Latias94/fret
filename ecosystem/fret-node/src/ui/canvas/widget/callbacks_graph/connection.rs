use super::super::*;

pub(super) fn emit_connection_callbacks(
    callbacks: &mut dyn NodeGraphCallbacks,
    committed: &GraphTransaction,
) {
    for change in connection_changes_from_transaction(committed) {
        callbacks.on_connection_change(change);
        match change {
            crate::runtime::callbacks::ConnectionChange::Connected(conn) => {
                callbacks.on_connect(conn)
            }
            crate::runtime::callbacks::ConnectionChange::Disconnected(conn) => {
                callbacks.on_disconnect(conn)
            }
            crate::runtime::callbacks::ConnectionChange::Reconnected { edge, from, to } => {
                callbacks.on_reconnect(edge, from, to);
                callbacks.on_edge_update(edge, from, to);
            }
        }
    }
}
