use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::state::ImUiPopupStore;

pub(super) fn prepare_popup_store_for_generation<H: UiHost>(
    store: &mut ImUiPopupStore,
    app: &mut H,
    window: AppWindowId,
    render_generation: u64,
) {
    let state = store.by_window.entry(window).or_default();
    let min_live_generation = render_generation.saturating_sub(1);
    for st in state.by_id.values_mut() {
        let is_open = app.models().get_copied(&st.open).unwrap_or(false);
        if !is_open {
            continue;
        }
        if st
            .keep_alive_generation
            .is_some_and(|generation| generation >= min_live_generation)
        {
            continue;
        }
        let _ = app.models_mut().update(&st.open, |v| *v = false);
        let _ = app.models_mut().update(&st.anchor, |v| *v = None);
        st.panel_id = None;
        st.keep_alive_generation = None;
    }
}
