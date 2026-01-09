use fret_core::Point;
use fret_ui::UiHost;

use super::super::state::{NodeResize, ViewSnapshot};
use super::NodeGraphCanvas;

pub(super) fn handle_pending_node_resize_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    _cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    _zoom: f32,
) -> bool {
    if canvas.interaction.node_resize.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_node_resize.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold.max(0.0);
    let dx = position.x.0 - pending.start_pos.x.0;
    let dy = position.y.0 - pending.start_pos.y.0;
    if threshold_screen > 0.0 && dx * dx + dy * dy < threshold_screen * threshold_screen {
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
