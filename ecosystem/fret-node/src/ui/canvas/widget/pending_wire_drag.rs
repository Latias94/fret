use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::super::state::{ViewSnapshot, WireDrag};
use super::NodeGraphCanvas;
use super::threshold::exceeds_drag_threshold;
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
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen) {
        return true;
    }

    canvas.interaction.pending_wire_drag = None;
    let kind = pending.kind.clone();
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: pending.kind,
        pos: pending.start_pos,
    });
    canvas.emit_connect_start(snapshot, &kind);

    wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
}
