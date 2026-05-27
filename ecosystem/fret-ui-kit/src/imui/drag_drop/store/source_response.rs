use fret_runtime::{DragKindId, Model};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::super::super::DragSourceResponse;
use super::state::ImUiDragDropStore;

pub(in crate::imui::drag_drop) fn source_response_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
) -> DragSourceResponse {
    let pointer_id = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            st.active
                .iter()
                .filter_map(|(session_id, active)| {
                    if active.source_id != trigger_id || active.kind != kind {
                        return None;
                    }
                    let drag = app.drag(active.pointer_id)?;
                    if drag.session_id != *session_id {
                        return None;
                    }
                    Some(active.pointer_id)
                })
                .min_by_key(|pointer_id| pointer_id.0)
        })
        .ok()
        .flatten();

    let Some(pointer_id) = pointer_id else {
        return DragSourceResponse::inactive();
    };
    let Some(drag) = cx.app.drag(pointer_id) else {
        return DragSourceResponse::inactive();
    };

    DragSourceResponse::new(
        drag.cross_window_hover,
        drag.position,
        pointer_id,
        drag.session_id,
    )
}
