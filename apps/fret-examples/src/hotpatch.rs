use fret_app::App;
use fret_core::AppWindowId;
use fret_ui::{InternalDragRouteService, UiTree};

pub(crate) fn hotpatch_drop_old_state() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("FRET_HOTPATCH_DROP_OLD_STATE").is_some_and(|v| !v.is_empty())
    }

    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

pub(crate) fn reset_ui_tree(app: &mut App, window: AppWindowId, ui: &mut UiTree<App>) {
    let mut new_ui: UiTree<App> = UiTree::new();
    new_ui.set_window(window);

    let old = std::mem::replace(ui, new_ui);
    if hotpatch_drop_old_state() {
        drop(old);
    } else {
        std::mem::forget(old);
    }

    app.with_global_mut(InternalDragRouteService::default, |svc, _app| {
        svc.clear_window(window);
    });
}
