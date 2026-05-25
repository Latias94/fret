use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use fret_core::{Point, PointerId};
use fret_runtime::{DragKindId, DragSessionId, Model, TickId};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::super::DragSourceResponse;

#[derive(Default)]
struct ImUiDragDropStoreGlobal {
    model: Option<Model<ImUiDragDropStore>>,
}

#[derive(Default)]
pub(super) struct ImUiDragDropStore {
    pub(super) active: HashMap<DragSessionId, ActiveDragPayload>,
    pub(super) delivered: HashMap<GlobalElementId, DeliveredDragPayload>,
}

#[derive(Clone)]
pub(super) struct ActiveDragPayload {
    pub(super) pointer_id: PointerId,
    pub(super) kind: DragKindId,
    pub(super) source_id: GlobalElementId,
    pub(super) hovered_target: Option<GlobalElementId>,
    pub(super) payload: Rc<dyn Any>,
}

#[derive(Clone)]
pub(super) struct DeliveredDragPayload {
    pub(super) tick_id: TickId,
    pub(super) session_id: DragSessionId,
    pub(super) source_id: GlobalElementId,
    pub(super) position: Point,
    pub(super) payload: Rc<dyn Any>,
}

pub(super) fn store_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ImUiDragDropStore> {
    cx.app
        .with_global_mut_untracked(ImUiDragDropStoreGlobal::default, |st, app| {
            if let Some(model) = st.model.clone() {
                return model;
            }

            let model = app.models_mut().insert(ImUiDragDropStore::default());
            st.model = Some(model.clone());
            model
        })
}

pub(super) fn prune_store<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
) {
    let current_tick = cx.app.tick_id();
    let stale_sessions = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            st.active
                .iter()
                .filter_map(|(session_id, active)| {
                    app.drag(active.pointer_id)
                        .filter(|drag| drag.session_id == *session_id && drag.kind == active.kind)
                        .map(|_| None)
                        .unwrap_or(Some(*session_id))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let _ = cx.app.models_mut().update(store, |st| {
        for session_id in &stale_sessions {
            st.active.remove(session_id);
        }

        st.delivered
            .retain(|_, delivery| current_tick.0 <= delivery.tick_id.0.saturating_add(1));
    });
}

pub(super) fn source_response_for<H: UiHost>(
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

pub(super) fn first_active_payload_for<T: Any, H: UiHost>(
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

pub(super) fn take_delivered_payload_for<T: Any, H: UiHost>(
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
