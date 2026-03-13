mod activate;
mod checks;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

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
    let pending = match checks::prepare_pending_wire_drag_move(canvas, snapshot, position, zoom) {
        checks::PendingWireDragMovePrep::NotHandled => return false,
        checks::PendingWireDragMovePrep::Handled => return true,
        checks::PendingWireDragMovePrep::Ready(pending) => pending,
    };

    activate::activate_pending_wire_drag(canvas, snapshot, pending);

    wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
}
