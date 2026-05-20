mod activate;
mod checks;

use fret_core::Point;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_pending_group_resize_move<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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

    if !checks::pending_group_resize_threshold_exceeded(&pending, snapshot, position, zoom) {
        return true;
    }

    activate::activate_pending_group_resize(&mut canvas.interaction, pending);

    false
}
