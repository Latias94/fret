use std::sync::Arc;

use fret_core::Rect;
use fret_runtime::Model;
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

#[cfg(feature = "shadcn-context-menu")]
#[derive(Debug, Default)]
struct WorkspaceTabStripContextMenuState {
    open: Option<Model<bool>>,
}

#[cfg(feature = "shadcn-context-menu")]
pub(super) fn get_context_menu_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    let existing = cx.with_state(WorkspaceTabStripContextMenuState::default, |st| {
        st.open.clone()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(false);
    cx.with_state(WorkspaceTabStripContextMenuState::default, |st| {
        st.open = Some(model.clone());
    });
    model
}
