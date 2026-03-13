mod coords;
mod internal_drop;
mod internal_move;
mod pending;
mod prelude;
mod session;

use prelude::*;

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

pub(super) fn handle_pending_insert_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: MouseButtons,
    zoom: f32,
) -> bool {
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

pub(super) fn handle_internal_drag_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    event: &InternalDragEvent,
    zoom: f32,
) -> bool {
    let pointer_id = event.pointer_id;
    let payload = cx
        .app
        .drag(pointer_id)
        .and_then(|d| d.payload::<InsertNodeDragPayload>())
        .cloned();
    let Some(payload) = payload else {
        session::clear_insert_node_drag_preview(&mut canvas.interaction, cx);
        return false;
    };

    match event.kind {
        InternalDragKind::Enter | InternalDragKind::Over => {
            internal_move::handle_enter_over(canvas, cx, snapshot, event, &payload, zoom)
        }
        InternalDragKind::Leave | InternalDragKind::Cancel => {
            session::clear_insert_node_drag_preview(&mut canvas.interaction, cx);
            session::finish_insert_node_drag_event(cx)
        }
        InternalDragKind::Drop => {
            internal_drop::handle_drop(canvas, cx, snapshot, event, payload, zoom)
        }
    }
}
