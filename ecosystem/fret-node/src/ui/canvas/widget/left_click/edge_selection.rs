use fret_ui::UiHost;

use crate::core::EdgeId;
use crate::io::NodeGraphViewState;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn edge_is_selectable<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    edge: EdgeId,
) -> bool {
    canvas
        .graph
        .read_ref(host, |graph| {
            NodeGraphCanvasWith::<M>::edge_is_selectable(graph, &snapshot.interaction, edge)
        })
        .ok()
        .unwrap_or(false)
}

pub(super) fn focused_edge_after_hit(
    edges_focusable: bool,
    edge_selectable: bool,
    edge: EdgeId,
) -> Option<EdgeId> {
    (edges_focusable && edge_selectable).then_some(edge)
}

pub(super) fn apply_edge_selection(
    view_state: &mut NodeGraphViewState,
    edge: EdgeId,
    multi_selection_pressed: bool,
) {
    if multi_selection_pressed {
        if let Some(index) = view_state.selected_edges.iter().position(|id| *id == edge) {
            view_state.selected_edges.remove(index);
        } else {
            view_state.selected_edges.push(edge);
        }
        return;
    }

    view_state.selected_nodes.clear();
    view_state.selected_groups.clear();
    view_state.selected_edges.clear();
    view_state.selected_edges.push(edge);
}

#[cfg(test)]
mod tests;
