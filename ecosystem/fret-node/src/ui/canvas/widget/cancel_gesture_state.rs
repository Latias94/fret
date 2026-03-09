use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::interaction::NodeGraphConnectionMode;
use crate::runtime::callbacks::{ConnectEndOutcome, NodeDragEndOutcome};

pub(super) fn cancel_gesture_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    mode: NodeGraphConnectionMode,
) -> bool {
    let mut canceled = false;

    if let Some(wire_drag) = canvas.interaction.wire_drag.take() {
        canvas.interaction.click_connect = false;
        canvas.emit_connect_end(mode, &wire_drag.kind, None, ConnectEndOutcome::Canceled);
        canceled = true;
    }
    canceled |= super::insert_node_drag::clear_insert_node_drag_state(&mut canvas.interaction);
    if canvas.interaction.edge_insert_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_edge_insert_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.edge_drag.take().is_some() {
        canceled = true;
    }
    if let Some(drag) = canvas.interaction.node_drag.take() {
        canvas.emit_node_drag_end(drag.primary, &drag.node_ids, NodeDragEndOutcome::Canceled);
        canceled = true;
    }
    if canvas.interaction.pending_node_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.group_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_group_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.group_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_group_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.node_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_node_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_wire_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.marquee.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_marquee.take().is_some() {
        canceled = true;
    }

    canceled
}
