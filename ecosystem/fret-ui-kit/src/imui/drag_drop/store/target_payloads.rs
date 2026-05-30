use std::any::Any;
use std::rc::Rc;

use fret_core::Point;
use fret_runtime::{DragSessionId, Model};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::state::ImUiDragDropStore;

pub(in crate::imui::drag_drop) fn first_active_payload_for<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
) -> Option<(DragSessionId, GlobalElementId, Point, Rc<T>)> {
    cx.read_model(store, Invalidation::Paint, |app, st| {
        st.active
            .iter()
            .filter_map(|(session_id, active)| {
                let drag = app.drag(active.pointer_id)?;
                if drag.session_id != *session_id || !drag.dragging {
                    return None;
                }
                let payload = active.payload.clone().downcast::<T>().ok()?;
                Some((
                    active.pointer_id,
                    drag.session_id,
                    active.source_id,
                    drag.position,
                    payload,
                ))
            })
            .min_by_key(|(pointer_id, _, _, _, _)| pointer_id.0)
            .map(|(_, session_id, source_id, position, payload)| {
                (session_id, source_id, position, payload)
            })
    })
    .ok()
    .flatten()
}

pub(in crate::imui::drag_drop) fn take_delivered_payload_for<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
    target_id: GlobalElementId,
) -> Option<(DragSessionId, GlobalElementId, Point, Rc<T>)> {
    let current_tick = cx.app.tick_id();
    cx.app
        .models_mut()
        .update(store, |st| {
            let delivered = st.delivered.remove(&target_id)?;
            if current_tick.0 > delivered.tick_id.0.saturating_add(1) {
                return None;
            }
            let payload = delivered.payload.downcast::<T>().ok()?;
            Some((
                delivered.session_id,
                delivered.source_id,
                delivered.position,
                payload,
            ))
        })
        .ok()
        .flatten()
}
