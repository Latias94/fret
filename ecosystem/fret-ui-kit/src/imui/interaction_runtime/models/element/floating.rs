use std::collections::HashMap;

use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Default)]
struct ImUiFloatWindowCollapsedStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<bool>>,
}

pub(in crate::imui) fn float_window_collapsed_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<bool> {
    cx.app
        .with_global_mut_untracked(ImUiFloatWindowCollapsedStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(false))
                .clone()
        })
}
