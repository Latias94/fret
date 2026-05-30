use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, Rect};
use fret_ui::{GlobalElementId, UiHost};

#[derive(Clone)]
pub(in crate::imui) struct PopupStoreState {
    pub(in crate::imui) open: fret_runtime::Model<bool>,
    pub(in crate::imui) anchor: fret_runtime::Model<Option<Rect>>,
    pub(in crate::imui) panel_id: Option<GlobalElementId>,
    /// Last IMUI render generation where the popup was "kept alive" by a `begin_popup_*` call.
    ///
    /// This is intentionally decoupled from the app's global `FrameId`: idle ticks can advance
    /// frame ids without any IMUI render pass, and open popups must not be treated as stale just
    /// because no redraw happened for a while.
    pub(in crate::imui) keep_alive_generation: Option<u64>,
}

impl PopupStoreState {
    pub(super) fn new<H: UiHost>(app: &mut H) -> Self {
        Self {
            open: app.models_mut().insert(false),
            anchor: app.models_mut().insert(None::<Rect>),
            panel_id: None,
            keep_alive_generation: None,
        }
    }
}

#[derive(Default)]
pub(super) struct PopupStoreWindowState {
    pub(super) by_id: HashMap<Arc<str>, PopupStoreState>,
}

#[derive(Default)]
pub(super) struct ImUiPopupStore {
    pub(super) by_window: HashMap<AppWindowId, PopupStoreWindowState>,
}
