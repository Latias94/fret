use fret_app::{App, Model};
use fret_kit::prelude::{
    InWindowMenubarFocusHandle, MenubarFromRuntimeOptions, menubar_from_runtime_with_focus_handle,
};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, Invalidation};
use std::cell::RefCell;
use std::sync::Arc;

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

    cx.app.with_global_mut(
        fret_runtime::WindowMenuBarFocusService::default,
        |svc, _app| {
            svc.set_present(cx.window, show_in_window_menu_bar && menu_bar.is_some());
        },
    );

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

    let group_active = handle.group_active.clone();
    let trigger_registry = handle.trigger_registry.clone();
    let last_focus_before_menubar = handle.last_focus_before_menubar.clone();
    let focus_is_trigger = handle.focus_is_trigger.clone();
    let group_active_for_command = group_active.clone();
    let trigger_registry_for_command = trigger_registry.clone();
    let last_focus_for_command = last_focus_before_menubar.clone();
    cx.command_add_on_command_for(
        panel_id,
        Arc::new(move |host, acx, command| {
            if command.as_str() != fret_app::core_commands::FOCUS_MENU_BAR {
                return false;
            }

            let active = host
                .models_mut()
                .get_cloned(&group_active_for_command)
                .flatten();
            if let Some(active) = active {
                let _ = host.models_mut().update(&active.open, |v| *v = false);
                let _ = host
                    .models_mut()
                    .update(&group_active_for_command, |v| *v = None);
                let restore = host
                    .models_mut()
                    .get_cloned(&last_focus_for_command)
                    .flatten();
                host.request_focus(restore.unwrap_or(active.trigger));
                host.request_redraw(acx.window);
                return true;
            }

            let entries = host
                .models_mut()
                .get_cloned(&trigger_registry_for_command)
                .unwrap_or_default();
            let target = entries.iter().find(|e| e.enabled).cloned();
            let Some(target) = target else {
                return false;
            };

            let open_for_state = target.open.clone();
            let _ = host.models_mut().update(&group_active_for_command, |v| {
                *v = Some(
                    fret_ui_kit::primitives::menubar::trigger_row::MenubarActiveTrigger {
                        trigger: target.trigger,
                        open: open_for_state,
                    },
                );
            });

            host.request_focus(target.trigger);
            host.request_redraw(acx.window);
            true
        }),
    );

    cx.key_add_on_key_down_for(
        panel_id,
        fret_ui_kit::primitives::menubar::trigger_row::open_on_alt_mnemonic(
            group_active.clone(),
            trigger_registry.clone(),
        ),
    );
    cx.key_add_on_key_down_for(
        panel_id,
        fret_ui_kit::primitives::menubar::trigger_row::open_on_mnemonic_when_active(
            group_active.clone(),
            trigger_registry.clone(),
            focus_is_trigger.clone(),
        ),
    );
    cx.key_add_on_key_down_for(
        panel_id,
        fret_ui_kit::primitives::menubar::trigger_row::exit_active_on_escape_when_closed(
            group_active.clone(),
            last_focus_before_menubar.clone(),
            focus_is_trigger.clone(),
        ),
    );
}
