use std::sync::Arc;

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

#[track_caller]
pub(super) fn get_focus_restore_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<WorkspaceTabStripPendingFocusRestore> {
    cx.local_model(WorkspaceTabStripPendingFocusRestore::default)
}

#[cfg(feature = "shadcn-context-menu")]
#[cfg(feature = "shadcn-context-menu")]
#[track_caller]
pub(super) fn get_context_menu_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    tab_id: &Arc<str>,
) -> Model<bool> {
    cx.local_model_keyed(tab_id.clone(), || false)
}

#[cfg(feature = "shadcn-context-menu")]
#[track_caller]
pub(super) fn get_overflow_menu_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    cx.local_model(|| false)
}

#[track_caller]
pub(super) fn get_reveal_hint_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pane_id: Option<&Arc<str>>,
) -> Model<WorkspaceTabStripRevealHint> {
    let key: Arc<str> = pane_id
        .cloned()
        .unwrap_or_else(|| Arc::<str>::from("workspace-tabstrip-default-pane"));
    cx.local_model_keyed(key, WorkspaceTabStripRevealHint::default)
}
