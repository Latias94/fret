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

pub(super) fn update_view_state_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&mut NodeGraphViewState),
) -> bool {
    let view_state = binding.view_state_model();
    let Ok(mut next_view_state) = host.models_mut().read(&view_state, |state| state.clone()) else {
        return false;
    };
    f(&mut next_view_state);

    binding
        .replace_view_state_action_host(host, next_view_state)
        .is_ok()
}

pub(super) fn update_view_state_ui_host<H: UiHost>(
    host: &mut H,
    view_state: &Model<NodeGraphViewState>,
    controller: &NodeGraphController,
    f: impl FnOnce(&mut NodeGraphViewState),
) -> bool {
    let Ok(mut next_view_state) = host.models_mut().read(view_state, |state| state.clone()) else {
        return false;
    };
    f(&mut next_view_state);

    controller
        .replace_view_state_and_sync_model(host, view_state, next_view_state)
        .is_ok()
}
