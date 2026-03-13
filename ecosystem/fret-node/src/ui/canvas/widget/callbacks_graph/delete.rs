use super::super::*;

pub(super) fn emit_delete_callbacks(
    callbacks: &mut dyn NodeGraphCallbacks,
    committed: &GraphTransaction,
) {
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
