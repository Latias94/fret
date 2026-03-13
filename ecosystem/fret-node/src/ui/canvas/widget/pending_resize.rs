mod activate;
mod checks;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_pending_node_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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

    if !checks::pending_node_resize_threshold_exceeded(&pending, snapshot, position, zoom) {
        return true;
    }

    activate::activate_pending_node_resize(&mut canvas.interaction, pending);

    false
}
