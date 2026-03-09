use fret_core::Point;
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;

use super::threshold::exceeds_drag_threshold;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingNodeSelectAction, ViewSnapshot};

pub(super) fn handle_pending_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.node_drag.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_node_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold;
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen, zoom) {
        return true;
    }

    if !pending.drag_enabled {
        return super::pending_drag_session::abort_pending_node_drag(&mut canvas.interaction, cx);
    }

    let primary_draggable = canvas
        .graph
        .read_ref(cx.app, |g| {
            NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, pending.primary)
        })
        .ok()
        .unwrap_or(false);
    if !primary_draggable {
        return super::pending_drag_session::abort_pending_node_drag(&mut canvas.interaction, cx);
    }

    if pending.select_action != PendingNodeSelectAction::None {
        let node = pending.primary;
        canvas.update_view_state(cx.app, |s| {
            let already_selected = s.selected_nodes.iter().any(|id| *id == node);
            if !already_selected {
                s.selected_nodes.push(node);
            }

            s.draw_order.retain(|id| *id != node);
            s.draw_order.push(node);
        });
    }

    let start_nodes = canvas
        .graph
        .read_ref(cx.app, |g| {
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
        return super::pending_drag_session::abort_pending_node_drag(&mut canvas.interaction, cx);
    }
    let drag_nodes: Vec<GraphNodeId> = start_nodes.iter().map(|(id, _)| *id).collect();
    let primary = pending.primary;
    super::pending_drag_session::activate_pending_node_drag(
        &mut canvas.interaction,
        pending,
        drag_nodes.clone(),
        start_nodes,
    );
    canvas.emit_node_drag_start(primary, &drag_nodes);

    false
}
