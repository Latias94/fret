use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::super::state::{ViewSnapshot, WireDrag};
use super::NodeGraphCanvas;
use super::wire_drag;

pub(super) fn handle_pending_wire_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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
    let dx = position.x.0 - pending.start_pos.x.0;
    let dy = position.y.0 - pending.start_pos.y.0;
    if threshold_screen > 0.0 && dx * dx + dy * dy < threshold_screen * threshold_screen {
        return true;
    }

    canvas.interaction.pending_wire_drag = None;
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: pending.kind,
        pos: pending.start_pos,
    });

    wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
}
