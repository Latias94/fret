use fret_runtime::Model;
use fret_ui::UiHost;

use super::compat_transport::NodeGraphEditQueue;
use super::controller::NodeGraphController;
use crate::core::Graph;
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;

fn enqueue_transaction<H: UiHost>(
    host: &mut H,
    edits: Option<&Model<NodeGraphEditQueue>>,
    tx: &GraphTransaction,
) {
    if let Some(edits) = edits {
        let _ = edits.update(host, |queue, _cx| {
            queue.push(tx.clone());
        });
    }
}

pub(crate) fn submit_graph_transaction<H: UiHost>(
    host: &mut H,
    controller: Option<&NodeGraphController>,
    edits: Option<&Model<NodeGraphEditQueue>>,
    graph: &Model<Graph>,
    tx: &GraphTransaction,
) {
    if let Some(controller) = controller {
        let _ = controller.submit_transaction_and_sync_graph_model(host, graph, tx);
        return;
    }

    enqueue_transaction(host, edits, tx);
}

pub(crate) fn submit_graph_and_view_transaction<H: UiHost>(
    host: &mut H,
    controller: Option<&NodeGraphController>,
    edits: Option<&Model<NodeGraphEditQueue>>,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    tx: &GraphTransaction,
) {
    if let Some(controller) = controller {
        let _ = controller.submit_transaction_and_sync_models(host, graph, view_state, tx);
        return;
    }

    enqueue_transaction(host, edits, tx);
}
