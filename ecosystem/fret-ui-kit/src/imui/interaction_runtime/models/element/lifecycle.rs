use std::collections::HashMap;

use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::ImUiLifecycleSessionState;

#[derive(Default)]
struct ImUiLifecycleSessionStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<ImUiLifecycleSessionState>>,
}

pub(in crate::imui) fn lifecycle_session_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<ImUiLifecycleSessionState> {
    cx.app
        .with_global_mut_untracked(ImUiLifecycleSessionStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| {
                    app.models_mut()
                        .insert(ImUiLifecycleSessionState::default())
                })
                .clone()
        })
}
