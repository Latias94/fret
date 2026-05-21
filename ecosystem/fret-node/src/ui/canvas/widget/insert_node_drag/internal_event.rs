use fret_ui::retained_bridge::EventCx;

use super::InsertNodeDragPayload;
use super::prelude::*;

pub(in super::super) fn handle_internal_drag_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
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
        super::session::clear_insert_node_drag_preview(&mut canvas.interaction, cx);
        return false;
    };

    match event.kind {
        InternalDragKind::Enter | InternalDragKind::Over => {
            super::internal_move::handle_enter_over(canvas, cx, snapshot, event, &payload, zoom)
        }
        InternalDragKind::Leave | InternalDragKind::Cancel => {
            super::session::clear_insert_node_drag_preview(&mut canvas.interaction, cx);
            super::session::finish_insert_node_drag_event(cx)
        }
        InternalDragKind::Drop => {
            super::internal_drop::handle_drop(canvas, cx, snapshot, event, payload, zoom)
        }
    }
}
