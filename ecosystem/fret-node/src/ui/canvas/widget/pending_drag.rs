use fret_core::Point;
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;

use super::threshold::exceeds_drag_threshold;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{NodeDrag, PendingNodeSelectAction, ViewSnapshot};

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
        canvas.interaction.pending_node_drag = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    let primary_draggable = canvas
        .graph
        .read_ref(cx.app, |g| {
            NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, pending.primary)
        })
        .ok()
        .unwrap_or(false);
    if !primary_draggable {
        canvas.interaction.pending_node_drag = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    canvas.interaction.pending_node_drag = None;

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
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    let drag_nodes: Vec<GraphNodeId> = start_nodes.iter().map(|(id, _)| *id).collect();
    canvas.interaction.node_drag = Some(NodeDrag {
        primary: pending.primary,
        node_ids: drag_nodes.clone(),
        nodes: start_nodes.clone(),
        current_nodes: start_nodes,
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: pending.grab_offset,
        start_pos: pending.start_pos,
    });
    canvas.emit_node_drag_start(pending.primary, &drag_nodes);

    false
}
