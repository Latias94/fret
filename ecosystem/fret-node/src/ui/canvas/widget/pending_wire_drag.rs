use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::threshold::exceeds_drag_threshold;
use super::wire_drag;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_pending_wire_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    if canvas.interaction.wire_drag.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_wire_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold.max(0.0);
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen, zoom) {
        return true;
    }

    let kind = super::pending_connection_session::activate_pending_wire_drag(
        &mut canvas.interaction,
        pending,
    );
    canvas.emit_connect_start(snapshot, &kind);

    wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
}
