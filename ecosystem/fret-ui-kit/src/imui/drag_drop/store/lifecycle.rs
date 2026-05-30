use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::state::{ImUiDragDropStore, ImUiDragDropStoreGlobal};

pub(in crate::imui::drag_drop) fn store_model_for<H: UiHost>(
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

pub(in crate::imui::drag_drop) fn prune_store<H: UiHost>(
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
