use std::collections::HashMap;

use fret_core::Point;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Default)]
struct ImUiContextMenuAnchorStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Option<Point>>>,
}

pub(in crate::imui) fn context_menu_anchor_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<Option<Point>> {
    cx.app
        .with_global_mut_untracked(ImUiContextMenuAnchorStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(None::<Point>))
                .clone()
        })
}
