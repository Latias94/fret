use std::sync::Arc;

use std::collections::HashMap;

use fret_core::Rect;
use fret_runtime::{Model, TimerToken};
use fret_ui::action::ActivateReason;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use crate::tab_drag::WorkspaceTabHitRect;

#[derive(Default)]
pub(super) struct WorkspaceTabStripState {
    pub(super) scroll: ScrollHandle,
    pub(super) last_active: Option<Arc<str>>,
    pub(super) reveal_pending: bool,
    pub(super) last_tab_rects: Vec<WorkspaceTabHitRect>,
    pub(super) last_scroll_viewport: Option<Rect>,
}

/// Best-effort hint for how the next selection change should reveal the active tab.
///
/// This is a policy-layer helper so pointer selection can avoid scroll-jank (no margin),
/// while keyboard/programmatic selection keeps comfortable context (margin).
#[derive(Debug, Default, Clone)]
pub(super) struct WorkspaceTabStripRevealHint {
    pub(super) tab_id: Option<Arc<str>>,
    pub(super) reason: Option<ActivateReason>,
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

#[derive(Default)]
struct WorkspaceTabStripRevealHintState {
    by_pane: HashMap<Arc<str>, Model<WorkspaceTabStripRevealHint>>,
}

pub(super) fn get_reveal_hint_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pane_id: Option<&Arc<str>>,
) -> Model<WorkspaceTabStripRevealHint> {
    let key: Arc<str> = pane_id
        .cloned()
        .unwrap_or_else(|| Arc::<str>::from("workspace-tabstrip-default-pane"));

    let existing = cx.with_state(WorkspaceTabStripRevealHintState::default, |st| {
        st.by_pane.get(key.as_ref()).cloned()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx
        .app
        .models_mut()
        .insert(WorkspaceTabStripRevealHint::default());
    cx.with_state(WorkspaceTabStripRevealHintState::default, |st| {
        st.by_pane.insert(key, model.clone());
    });
    model
}
