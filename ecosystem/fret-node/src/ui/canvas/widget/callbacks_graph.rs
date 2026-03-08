use super::*;

pub(super) fn emit_graph_callbacks<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    committed: &GraphTransaction,
    changes: &NodeGraphChanges,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };

    callbacks.on_graph_commit(committed, changes);
    if !changes.nodes.is_empty() {
        callbacks.on_nodes_change(&changes.nodes);
    }
    if !changes.edges.is_empty() {
        callbacks.on_edges_change(&changes.edges);
    }
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

    let deleted = crate::runtime::callbacks::delete_changes_from_transaction(committed);
    if !deleted.nodes.is_empty() {
        callbacks.on_nodes_delete(&deleted.nodes);
    }
    if !deleted.edges.is_empty() {
        callbacks.on_edges_delete(&deleted.edges);
    }
    if !deleted.groups.is_empty() {
        callbacks.on_groups_delete(&deleted.groups);
    }
    if !deleted.sticky_notes.is_empty() {
        callbacks.on_sticky_notes_delete(&deleted.sticky_notes);
    }
    if !deleted.nodes.is_empty()
        || !deleted.edges.is_empty()
        || !deleted.groups.is_empty()
        || !deleted.sticky_notes.is_empty()
    {
        callbacks.on_delete(deleted);
    }
}
