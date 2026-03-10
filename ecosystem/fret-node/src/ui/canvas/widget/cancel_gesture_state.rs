mod sessions;
mod wire;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::interaction::NodeGraphConnectionMode;
use crate::runtime::callbacks::NodeDragEndOutcome;

pub(super) fn cancel_gesture_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    mode: NodeGraphConnectionMode,
) -> bool {
    let mut canceled = false;

    canceled |= wire::cancel_wire_gesture_state(canvas, mode);

    if let Some(drag) = canvas.interaction.node_drag.take() {
        canvas.emit_node_drag_end(drag.primary, &drag.node_ids, NodeDragEndOutcome::Canceled);
        canceled = true;
    }

    canceled |= sessions::clear_remaining_gesture_sessions(&mut canvas.interaction);

    canceled
}
