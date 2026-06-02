use std::collections::HashMap;

use fret_core::Modifiers;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::LongPressSignalState;

#[derive(Default)]
struct ImUiLongPressStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<LongPressSignalState>>,
}

#[derive(Default)]
struct ImUiPointerClickModifiersStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Modifiers>>,
}

pub(in crate::imui) fn long_press_signal_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<LongPressSignalState> {
    cx.app
        .with_global_mut_untracked(ImUiLongPressStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(LongPressSignalState::default()))
                .clone()
        })
}

pub(in crate::imui) fn pointer_click_modifiers_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<Modifiers> {
    cx.app
        .with_global_mut_untracked(ImUiPointerClickModifiersStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(Modifiers::default()))
                .clone()
        })
}
