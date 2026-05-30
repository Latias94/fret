use std::collections::HashMap;

use fret_core::{Modifiers, Point};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ImUiLifecycleSessionState, LongPressSignalState};

#[derive(Default)]
struct ImUiContextMenuAnchorStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Option<Point>>>,
}

#[derive(Default)]
struct ImUiLongPressStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<LongPressSignalState>>,
}

#[derive(Default)]
struct ImUiPointerClickModifiersStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Modifiers>>,
}

#[derive(Default)]
struct ImUiLifecycleSessionStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<ImUiLifecycleSessionState>>,
}

#[derive(Default)]
struct ImUiFloatWindowCollapsedStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<bool>>,
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
