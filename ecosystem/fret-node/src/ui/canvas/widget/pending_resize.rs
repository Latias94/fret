use fret_canvas::drag::DragThreshold;
use fret_core::Point;
use fret_ui::UiHost;

use super::super::state::{NodeResize, ViewSnapshot};
use super::NodeGraphCanvas;

pub(super) fn handle_pending_node_resize_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    _cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.node_resize.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_node_resize.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold.max(0.0);
    let threshold_graph = DragThreshold {
        screen_px: threshold_screen,
    }
    .to_canvas_units(zoom);
    let dx = position.x.0 - pending.start_pos.x.0;
    let dy = position.y.0 - pending.start_pos.y.0;
    if threshold_graph > 0.0 && dx * dx + dy * dy < threshold_graph * threshold_graph {
        return true;
    }

    canvas.interaction.pending_node_resize = None;
    canvas.interaction.node_resize = Some(NodeResize {
        node: pending.node,
        start_pos: pending.start_pos,
        start_size: pending.start_size,
        start_size_opt: pending.start_size_opt,
    });

    false
}
