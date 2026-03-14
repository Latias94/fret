use fret::workspace_menu::{
    InWindowMenubarFocusHandle, MenubarFromRuntimeOptions, install_in_window_menubar_focus_bridge,
    menubar_from_runtime_with_focus_handle,
};
use fret_app::{App, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, Invalidation};
use std::cell::RefCell;

pub(super) fn build_in_window_menu_bar(
    cx: &mut ElementContext<'_, App>,
    menu_bar_seq: &Model<u64>,
    menubar_handle: &RefCell<Option<InWindowMenubarFocusHandle>>,
) -> Vec<AnyElement> {
    let menu_bar_seq_value = cx
        .get_model_copied(menu_bar_seq, Invalidation::Layout)
        .unwrap_or(0);
    let menu_bar = fret_app::effective_menu_bar(cx.app);
    let show_in_window_menu_bar =
        fret_app::should_render_in_window_menu_bar(cx.app, fret_app::Platform::current());

    let present = show_in_window_menu_bar && menu_bar.is_some();
    let needs_update = cx
        .app
        .global::<fret_runtime::WindowMenuBarFocusService>()
        .is_none_or(|svc| svc.present(cx.window) != present);
    if needs_update {
        cx.app.with_global_mut(
            fret_runtime::WindowMenuBarFocusService::default,
            |svc, _app| {
                svc.set_present(cx.window, present);
            },
        );
    }

    if !show_in_window_menu_bar {
        return Vec::new();
    }

    menu_bar
        .as_ref()
        .map(|menu_bar| {
            cx.keyed(format!("ui_gallery.menubar.{menu_bar_seq_value}"), |cx| {
                let (menu, handle) = menubar_from_runtime_with_focus_handle(
                    cx,
                    menu_bar,
                    MenubarFromRuntimeOptions::default(),
                );
                *menubar_handle.borrow_mut() = Some(handle);
                menu
            })
        })
        .into_iter()
        .collect()
}

pub(super) fn attach_in_window_menubar_handlers(
    cx: &mut ElementContext<'_, App>,
    panel_id: GlobalElementId,
    menubar_handle: &RefCell<Option<InWindowMenubarFocusHandle>>,
) {
    let Some(handle) = menubar_handle.borrow().clone() else {
        return;
    };
    install_in_window_menubar_focus_bridge(cx, panel_id, &handle);
}
