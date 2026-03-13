mod connection;
mod delete;

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

    connection::emit_connection_callbacks(callbacks.as_mut(), committed);
    delete::emit_delete_callbacks(callbacks.as_mut(), committed);
}
