#[cfg(feature = "shadcn-context-menu")]
use std::collections::HashMap;
use std::sync::Arc;

use fret_core::Rect;
use fret_runtime::{Model, TimerToken};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use crate::tab_drag::WorkspaceTabHitRect;

#[derive(Default)]
pub(super) struct WorkspaceTabStripState {
    pub(super) scroll: ScrollHandle,
    pub(super) last_active: Option<Arc<str>>,
    pub(super) last_tab_rects: Vec<WorkspaceTabHitRect>,
    pub(super) last_scroll_viewport: Option<Rect>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct WorkspaceTabStripPendingFocusRestore {
    pub(super) timer: Option<TimerToken>,
    pub(super) target_pane_id: Option<Arc<str>>,
    pub(super) tab_id: Option<Arc<str>>,
    pub(super) attempts: u32,
}

#[derive(Default)]
struct WorkspaceTabStripFocusRestoreModelState {
    model: Option<Model<WorkspaceTabStripPendingFocusRestore>>,
}

pub(super) fn get_focus_restore_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<WorkspaceTabStripPendingFocusRestore> {
    let existing = cx.with_state(WorkspaceTabStripFocusRestoreModelState::default, |st| {
        st.model.clone()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx
        .app
        .models_mut()
        .insert(WorkspaceTabStripPendingFocusRestore::default());
    cx.with_state(WorkspaceTabStripFocusRestoreModelState::default, |st| {
        st.model = Some(model.clone());
    });
    model
}

#[cfg(feature = "shadcn-context-menu")]
#[derive(Debug, Default)]
struct WorkspaceTabStripContextMenuState {
    open_by_tab: HashMap<Arc<str>, Model<bool>>,
}

#[cfg(feature = "shadcn-context-menu")]
pub(super) fn get_context_menu_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    tab_id: &Arc<str>,
) -> Model<bool> {
    let existing = cx.with_state(WorkspaceTabStripContextMenuState::default, |st| {
        st.open_by_tab.get(tab_id).cloned()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(false);
    let tab_id = tab_id.clone();
    cx.with_state(WorkspaceTabStripContextMenuState::default, |st| {
        st.open_by_tab.insert(tab_id, model.clone());
    });
    model
}

#[cfg(feature = "shadcn-context-menu")]
#[derive(Debug, Default)]
struct WorkspaceTabStripOverflowMenuState {
    open: Option<Model<bool>>,
}

#[cfg(feature = "shadcn-context-menu")]
pub(super) fn get_overflow_menu_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    let existing = cx.with_state(WorkspaceTabStripOverflowMenuState::default, |st| {
        st.open.clone()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(false);
    cx.with_state(WorkspaceTabStripOverflowMenuState::default, |st| {
        st.open = Some(model.clone());
    });
    model
}
