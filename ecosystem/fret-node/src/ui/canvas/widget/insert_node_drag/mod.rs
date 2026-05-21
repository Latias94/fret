mod coords;
mod internal_drop;
mod internal_event;
mod internal_move;
mod pending;
mod prelude;
mod session;

use prelude::*;

pub(super) use internal_event::handle_internal_drag_event;

/// Payload type for "drag a node from the palette/searcher into the canvas".
#[derive(Debug, Clone)]
pub(super) struct InsertNodeDragPayload {
    pub(super) candidate: InsertNodeCandidate,
}

pub(super) const DRAG_KIND_INSERT_NODE: DragKindId = DragKindId(0x4E4F44455F494E53);

pub(super) fn clear_insert_node_drag_state(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    session::clear_insert_node_drag_state(interaction)
}

pub(super) fn handle_pending_insert_node_drag_move<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: MouseButtons,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: InsertNodeDragMoveCx<H>,
{
    pending::handle_pending_insert_node_drag_move(
        canvas,
        cx,
        snapshot,
        position,
        buttons,
        zoom,
        DRAG_KIND_INSERT_NODE,
    )
}
