use fret_core::Point;
use fret_ui::UiHost;

use super::super::state::{NodeDrag, ViewSnapshot};
use super::NodeGraphCanvas;

pub(super) fn handle_pending_node_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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

    let threshold_screen = snapshot.interaction.node_drag_threshold.max(0.0);
    let threshold_graph = threshold_screen / zoom;
    let dx = position.x.0 - pending.start_pos.x.0;
    let dy = position.y.0 - pending.start_pos.y.0;
    if threshold_graph > 0.0 && dx * dx + dy * dy < threshold_graph * threshold_graph {
        return true;
    }

    canvas.interaction.pending_node_drag = None;
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
