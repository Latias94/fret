use super::*;
use crate::ops::{GraphOp, normalize_transaction};

pub(super) fn build_node_drag_transaction(
    graph: &Graph,
    nodes: &[crate::core::NodeId],
    dx: f32,
    dy: f32,
) -> GraphTransaction {
    let mut tx = GraphTransaction::new();
    for id in nodes.iter().copied() {
        let Some(node) = graph.nodes.get(&id) else {
            continue;
        };
        let from = node.pos;
        let to = crate::core::CanvasPoint {
            x: from.x + dx,
            y: from.y + dy,
        };
        if from != to {
            tx.push(GraphOp::SetNodePos { id, from, to });
        }
    }

    let tx = normalize_transaction(tx);
    if tx.is_empty() {
        return tx;
    }

    let label = if tx.ops.len() == 1 {
        "Move Node"
    } else {
        "Move Nodes"
    };
    tx.with_label(label)
}

pub(super) fn commit_graph_transaction(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    tx: &GraphTransaction,
) -> bool {
    if tx.is_empty() {
        return true;
    }

    binding.dispatch_transaction_action_host(host, tx).is_ok()
}

pub(super) fn commit_node_drag_transaction(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    tx: &GraphTransaction,
) -> bool {
    commit_graph_transaction(host, binding, tx)
}

pub(super) fn authoritative_graph_snapshot_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
) -> Option<Graph> {
    let store = binding.store_model();
    host.models_mut()
        .read(&store, |store| store.graph().clone())
        .ok()
}

pub(super) fn update_view_state_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&mut NodeGraphViewState),
) -> bool {
    let store = binding.store_model();
    if host
        .models_mut()
        .update(&store, move |store| store.update_view_state(f))
        .is_err()
    {
        return false;
    }
    let _ = binding.sync_from_store_action_host(host);
    true
}

pub(super) fn update_view_state_ui_host<H: UiHost>(
    host: &mut H,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&mut NodeGraphViewState),
) -> bool {
    let store = binding.store_model();
    if host
        .models_mut()
        .update(&store, move |store| store.update_view_state(f))
        .is_err()
    {
        return false;
    }
    let _ = binding.sync_from_store(host);
    true
}
