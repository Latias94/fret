use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use fret_core::{AppWindowId, Modifiers, Point};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in super::super) struct LongPressSignalState {
    pub(in super::super) timer: Option<fret_runtime::TimerToken>,
    pub(in super::super) holding: bool,
}

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in super::super) struct ImUiLifecycleSessionState {
    pub(in super::super) active: bool,
    pub(in super::super) edited_during_active: bool,
}

#[derive(Default)]
struct ImUiLifecycleSessionStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<ImUiLifecycleSessionState>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in super::super) struct ImUiActiveItemState {
    pub(in super::super) active: Option<GlobalElementId>,
}

#[derive(Default)]
struct ImUiActiveItemStore {
    by_window: HashMap<AppWindowId, fret_runtime::Model<ImUiActiveItemState>>,
}

#[derive(Default)]
struct ImUiFloatWindowCollapsedStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<bool>>,
}

#[derive(Default)]
struct ImUiDisabledScopeStore {
    depth: Rc<Cell<u32>>,
}

pub(in super::super) fn context_menu_anchor_model_for<H: UiHost>(
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

pub(in super::super) fn long_press_signal_model_for<H: UiHost>(
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

pub(in super::super) fn pointer_click_modifiers_model_for<H: UiHost>(
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

pub(in super::super) fn lifecycle_session_model_for<H: UiHost>(
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

pub(in super::super) fn active_item_model_for_window<H: UiHost>(
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

pub(in super::super) fn float_window_collapsed_model_for<H: UiHost>(
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

pub(in super::super) fn disabled_scope_depth_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Rc<Cell<u32>> {
    cx.app
        .with_global_mut_untracked(ImUiDisabledScopeStore::default, |st, _app| st.depth.clone())
}
