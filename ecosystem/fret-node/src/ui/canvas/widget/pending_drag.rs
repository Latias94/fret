mod activate;
mod checks;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

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

    if !checks::pending_drag_threshold_exceeded(&pending, snapshot, position, zoom) {
        return true;
    }

    if !pending.drag_enabled {
        return super::pending_drag_session::abort_pending_node_drag(&mut canvas.interaction, cx);
    }

    if !checks::primary_node_is_draggable(canvas, cx.app, snapshot, pending.primary) {
        return super::pending_drag_session::abort_pending_node_drag(&mut canvas.interaction, cx);
    }

    let Some((drag_nodes, start_nodes)) =
        activate::drag_start_nodes(canvas, cx.app, snapshot, &pending)
    else {
        return super::pending_drag_session::abort_pending_node_drag(&mut canvas.interaction, cx);
    };

    activate::apply_pending_selection(canvas, cx.app, &pending);
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
