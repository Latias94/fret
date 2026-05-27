mod connection;
mod delete;

use super::*;

pub(super) fn emit_graph_callbacks<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    patch: &NodeGraphPatch,
    node_edge_changes: &NodeGraphChanges,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };

    callbacks.on_graph_commit(patch);
    callbacks.on_node_edge_changes(node_edge_changes);
    if !node_edge_changes.nodes.is_empty() {
        callbacks.on_nodes_change(&node_edge_changes.nodes);
    }
    if !node_edge_changes.edges.is_empty() {
        callbacks.on_edges_change(&node_edge_changes.edges);
    }

    connection::emit_connection_callbacks(callbacks.as_mut(), patch.transaction());
    delete::emit_delete_callbacks(callbacks.as_mut(), patch.transaction());
}
