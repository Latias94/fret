use std::collections::HashMap;

use fret_core::AppWindowId;
use fret_ui::{ElementContext, UiHost};

use super::ImUiActiveItemState;

#[derive(Default)]
struct ImUiActiveItemStore {
    by_window: HashMap<AppWindowId, fret_runtime::Model<ImUiActiveItemState>>,
}

pub(in crate::imui) fn active_item_model_for_window<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_runtime::Model<ImUiActiveItemState> {
    let window = cx.window;
    cx.app
        .with_global_mut_untracked(ImUiActiveItemStore::default, |st, app| {
            st.by_window
                .entry(window)
                .or_insert_with(|| app.models_mut().insert(ImUiActiveItemState::default()))
                .clone()
        })
}
