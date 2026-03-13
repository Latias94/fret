use fret_ui::UiHost;

use crate::core::{EdgeId, NodeId as GraphNodeId};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn apply_marquee_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    selected_nodes: Vec<GraphNodeId>,
    selected_edges: Vec<EdgeId>,
) {
    canvas.update_view_state(host, |state| {
        state.selected_edges.clear();
        state.selected_groups.clear();
        state.selected_nodes = selected_nodes;
        state.selected_edges = selected_edges;
    });
}
