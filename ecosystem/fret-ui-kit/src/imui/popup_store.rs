//! Internal popup-scope state storage for immediate-mode helpers.

use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::FrameId;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Clone)]
pub(super) struct PopupStoreState {
    pub(super) open: fret_runtime::Model<bool>,
    pub(super) anchor: fret_runtime::Model<Option<fret_core::Rect>>,
    pub(super) panel_id: Option<GlobalElementId>,
    /// Last frame id where the popup was "kept alive" by a `begin_popup_*` call.
    pub(super) keep_alive_frame: Option<FrameId>,
}

#[derive(Default)]
struct PopupStoreWindowState {
    by_id: HashMap<Arc<str>, PopupStoreState>,
    prepared_frame: Option<FrameId>,
}

#[derive(Default)]
struct ImUiPopupStore {
    by_window: HashMap<AppWindowId, PopupStoreWindowState>,
}

fn prepare_popup_store_for_frame<H: UiHost>(
    store: &mut ImUiPopupStore,
    app: &mut H,
    window: AppWindowId,
    frame_id: FrameId,
) {
    let state = store.by_window.entry(window).or_default();
    if state.prepared_frame == Some(frame_id) {
        return;
    }
    state.prepared_frame = Some(frame_id);

    let required_keep_alive = FrameId(frame_id.0.saturating_sub(1));
    for st in state.by_id.values_mut() {
        let is_open = app.models().get_copied(&st.open).unwrap_or(false);
        if !is_open {
            continue;
        }
        if st.keep_alive_frame == Some(required_keep_alive) {
            continue;
        }
        let _ = app.models_mut().update(&st.open, |v| *v = false);
        let _ = app.models_mut().update(&st.anchor, |v| *v = None);
        st.panel_id = None;
    }
}

pub(super) fn with_popup_store_for_id<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    f: impl FnOnce(&mut PopupStoreState, &mut H) -> R,
) -> R {
    let window = cx.window;
    let frame_id = cx.frame_id;
    cx.app
        .with_global_mut_untracked(ImUiPopupStore::default, |store, app| {
            prepare_popup_store_for_frame(store, app, window, frame_id);

            let state = store.by_window.entry(window).or_default();
            if let Some(existing) = state.by_id.get_mut(id) {
                return f(existing, app);
            }

            let key: Arc<str> = Arc::from(id);
            let entry = state.by_id.entry(key).or_insert_with(|| PopupStoreState {
                open: app.models_mut().insert(false),
                anchor: app.models_mut().insert(None::<fret_core::Rect>),
                panel_id: None,
                keep_alive_frame: None,
            });
            f(entry, app)
        })
}

pub(super) fn drop_popup_scope_for_id<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &str) {
    cx.app
        .with_global_mut_untracked(ImUiPopupStore::default, |store, app| {
            prepare_popup_store_for_frame(store, app, cx.window, cx.frame_id);
            let Some(window_state) = store.by_window.get_mut(&cx.window) else {
                return;
            };
            let Some(entry) = window_state.by_id.remove(id) else {
                return;
            };
            let _ = app.models_mut().update(&entry.open, |v| *v = false);
            let _ = app.models_mut().update(&entry.anchor, |v| *v = None);
        });
    cx.app.request_redraw(cx.window);
}
