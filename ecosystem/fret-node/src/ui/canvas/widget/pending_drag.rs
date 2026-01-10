use fret_core::Point;
use fret_ui::UiHost;

use super::super::state::{NodeDrag, PendingNodeSelectAction, ViewSnapshot};
use super::NodeGraphCanvas;
use super::threshold::exceeds_drag_threshold;

pub(super) fn handle_pending_node_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    _zoom: f32,
) -> bool {
    if canvas.interaction.node_drag.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_node_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold;
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen) {
        return true;
    }

    if !pending.drag_enabled {
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
            s.selected_edges.clear();
            s.selected_groups.clear();

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
                .filter_map(|id| g.nodes.get(&id).map(|n| (id, n.pos)))
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default();
    canvas.interaction.node_drag = Some(NodeDrag {
        primary: pending.primary,
        nodes: start_nodes,
        grab_offset: pending.grab_offset,
        start_pos: pending.start_pos,
    });

    false
}
