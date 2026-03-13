use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::interaction::NodeGraphConnectionMode;
use crate::runtime::callbacks::ConnectEndOutcome;

pub(in super::super) fn cancel_wire_gesture_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    mode: NodeGraphConnectionMode,
) -> bool {
    let Some(wire_drag) = canvas.interaction.wire_drag.take() else {
        return false;
    };

    canvas.interaction.click_connect = false;
    canvas.emit_connect_end(mode, &wire_drag.kind, None, ConnectEndOutcome::Canceled);
    true
}
