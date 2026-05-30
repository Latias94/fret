use fret_authoring::mark_immediate_render_frame;
use fret_ui::{ElementContext, UiHost};

use super::lifecycle::prepare_popup_store_for_generation;
use super::state::ImUiPopupStore;

pub(in crate::imui) fn drop_popup_scope_for_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
) {
    let render_generation = mark_immediate_render_frame(cx);
    cx.app
        .with_global_mut_untracked(ImUiPopupStore::default, |store, app| {
            prepare_popup_store_for_generation(store, app, cx.window, render_generation);
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
