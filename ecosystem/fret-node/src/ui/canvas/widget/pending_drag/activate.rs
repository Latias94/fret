use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};
use crate::ui::canvas::state::{PendingNodeDrag, PendingNodeSelectAction, ViewSnapshot};
use crate::ui::canvas::widget::*;

pub(super) fn apply_pending_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    pending: &PendingNodeDrag,
) {
    if pending.select_action != PendingNodeSelectAction::None {
        let node = pending.primary;
        canvas.update_view_state(host, |s| {
            let already_selected = s.selected_nodes.iter().any(|id| *id == node);
            if !already_selected {
                s.selected_nodes.push(node);
            }

            s.draw_order.retain(|id| *id != node);
            s.draw_order.push(node);
        });
    }
}

pub(super) fn drag_start_nodes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    pending: &PendingNodeDrag,
) -> Option<(Vec<GraphNodeId>, Vec<(GraphNodeId, CanvasPoint)>)> {
    let start_nodes = canvas
        .graph
        .read_ref(host, |g| {
            pending
                .nodes
                .iter()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, *id)
                })
                .filter_map(|id| g.nodes.get(&id).map(|n| (id, n.pos)))
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default();
    if start_nodes.is_empty() {
        return None;
    }
    let drag_nodes: Vec<GraphNodeId> = start_nodes.iter().map(|(id, _)| *id).collect();
    Some((drag_nodes, start_nodes))
}
