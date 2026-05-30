use std::sync::Arc;

use fret_authoring::mark_immediate_render_frame;
use fret_ui::{ElementContext, UiHost};

use super::lifecycle::prepare_popup_store_for_generation;
use super::state::{ImUiPopupStore, PopupStoreState};

pub(in crate::imui) fn popup_render_generation_for_window<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> u64 {
    let window = cx.window;
    let render_generation = mark_immediate_render_frame(cx);
    cx.app
        .with_global_mut_untracked(ImUiPopupStore::default, |store, app| {
            prepare_popup_store_for_generation(store, app, window, render_generation);
            render_generation
        })
}

pub(in crate::imui) fn with_popup_store_for_id<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    f: impl FnOnce(&mut PopupStoreState, &mut H) -> R,
) -> R {
    let window = cx.window;
    let render_generation = mark_immediate_render_frame(cx);
    cx.app
        .with_global_mut_untracked(ImUiPopupStore::default, |store, app| {
            prepare_popup_store_for_generation(store, app, window, render_generation);

            let state = store.by_window.entry(window).or_default();
            if let Some(existing) = state.by_id.get_mut(id) {
                return f(existing, app);
            }

            let key: Arc<str> = Arc::from(id);
            let entry = state
                .by_id
                .entry(key)
                .or_insert_with(|| PopupStoreState::new(app));
            f(entry, app)
        })
}
