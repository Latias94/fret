use fret_core::Point;
use fret_ui::UiHost;

use super::threshold::exceeds_drag_threshold;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_pending_group_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    _cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.group_resize.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_group_resize.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold;
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen, zoom) {
        return true;
    }

    super::pending_resize_session::activate_pending_group_resize(&mut canvas.interaction, pending);

    false
}
