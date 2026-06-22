use std::collections::HashMap;

use fret_core::PointerId;
use fret_runtime::{DragKindId, DragSessionId, Model, TickId};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::super::ColorEditDragDropPayload;

#[derive(Default)]
struct ColorDragDropStoreGlobal {
    model: Option<Model<ColorDragDropStore>>,
}

#[derive(Default)]
pub(in crate::controls::color_edit) struct ColorDragDropStore {
    pub(super) active: HashMap<DragSessionId, ActiveColorDrag>,
    pub(super) delivered: HashMap<GlobalElementId, DeliveredColorDrop>,
}

#[derive(Clone, Copy)]
pub(in crate::controls::color_edit) struct ActiveColorDrag {
    pub(super) pointer_id: PointerId,
    pub(super) kind: DragKindId,
    pub(super) source_id: GlobalElementId,
    pub(super) hovered_target: Option<GlobalElementId>,
    pub(super) payload: ColorEditDragDropPayload,
}

#[derive(Clone, Copy)]
pub(in crate::controls::color_edit) struct DeliveredColorDrop {
    pub(super) tick_id: TickId,
    pub(super) payload: ColorEditDragDropPayload,
}

pub(in crate::controls::color_edit) fn color_drag_drop_store_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ColorDragDropStore> {
    cx.app
        .with_global_mut_untracked(ColorDragDropStoreGlobal::default, |st, app| {
            if let Some(model) = st.model.clone() {
                return model;
            }

            let model = app.models_mut().insert(ColorDragDropStore::default());
            st.model = Some(model.clone());
            model
        })
}

pub(in crate::controls::color_edit) fn prune_color_drag_drop_store<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
) {
    let current_tick = cx.app.tick_id();
    let (stale_sessions, has_stale_delivered) = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            let stale_sessions = st
                .active
                .iter()
                .filter_map(|(session_id, active)| {
                    app.drag(active.pointer_id)
                        .filter(|drag| drag.session_id == *session_id && drag.kind == active.kind)
                        .map(|_| None)
                        .unwrap_or(Some(*session_id))
                })
                .collect::<Vec<_>>();
            let has_stale_delivered = st
                .delivered
                .values()
                .any(|drop| current_tick.0 > drop.tick_id.0.saturating_add(1));
            (stale_sessions, has_stale_delivered)
        })
        .unwrap_or_default();

    if stale_sessions.is_empty() && !has_stale_delivered {
        return;
    }

    let _ = cx.app.models_mut().update(store, |st| {
        for session_id in &stale_sessions {
            st.active.remove(session_id);
        }

        st.delivered
            .retain(|_, drop| current_tick.0 <= drop.tick_id.0.saturating_add(1));
    });
}
