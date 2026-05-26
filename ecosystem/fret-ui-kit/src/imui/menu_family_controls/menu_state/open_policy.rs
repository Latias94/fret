use std::sync::Arc;

use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

use super::capture::{BeginMenuState, read_bool_model, read_open_menu_model};

pub(in crate::imui::menu_family_controls) fn sync_open_menu_for_active_trigger<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_after_trigger: bool,
    trigger_clicked: bool,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(policy) = state.menubar_policy.as_ref() else {
        return;
    };
    if !open_after_trigger || trigger_clicked {
        return;
    }
    let Some(trigger_id) = trigger_id else {
        return;
    };

    let is_active_trigger = ui.with_cx_mut(|cx| {
        cx.read_model(
            &policy.group_active,
            fret_ui::Invalidation::Paint,
            |_app, value| {
                value
                    .as_ref()
                    .is_some_and(|active| active.trigger == trigger_id)
            },
        )
        .unwrap_or(false)
    });
    if is_active_trigger {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&policy.open_menu, |value| *value = Some(Arc::from(id)));
        });
    }
}

pub(in crate::imui::menu_family_controls) fn reconcile_menubar_after_trigger<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_after_trigger: bool,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(policy) = state.menubar_policy.as_ref() else {
        return;
    };

    if open_after_trigger && !state.popup_open_before && state.was_popup_open_before_render {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&state.row_open, |value| *value = false);
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if value.as_ref().is_some_and(|current| current.as_ref() == id) {
                    *value = None;
                }
            });
            if let Some(trigger_id) = trigger_id {
                let _ = cx.app.models_mut().update(&policy.group_active, |value| {
                    if value
                        .as_ref()
                        .is_some_and(|active| active.trigger == trigger_id)
                    {
                        *value = None;
                    }
                });
            }
        });
    }
    if !state.open_before && state.was_open_before_render {
        if state.popup_open_before {
            ui.close_popup(id);
        }
        ui.with_cx_mut(|cx| {
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if value.as_ref().is_some_and(|current| current.as_ref() == id) {
                    *value = None;
                }
            });
        });
    }
}

pub(in crate::imui::menu_family_controls) fn toggle_menu_on_trigger_click<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
) {
    if let Some(policy) = state.menubar_policy.as_ref() {
        ui.with_cx_mut(|cx| {
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if state.open_before && value.as_ref().is_some_and(|current| current.as_ref() == id)
                {
                    *value = None;
                } else {
                    *value = Some(Arc::from(id));
                }
            });
        });
    } else if state.open_before {
        ui.close_popup(id);
    }
}

pub(in crate::imui::menu_family_controls) fn resolve_open_requested<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_menu_before: Option<Arc<str>>,
) -> bool {
    let Some(policy) = state.menubar_policy.as_ref() else {
        return read_bool_model(ui, &state.popup_open);
    };

    let open_menu_now = read_open_menu_model(ui, policy);
    let should_close = state.open_before
        && (open_menu_now
            .as_ref()
            .is_some_and(|current| current.as_ref() != id)
            || (open_menu_before
                .as_ref()
                .is_some_and(|current| current.as_ref() == id)
                && open_menu_now.is_none()));
    if should_close {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&state.row_open, |value| *value = false);
        });
        ui.close_popup(id);
    }
    let requested_by_policy = open_menu_now
        .as_ref()
        .is_some_and(|current| current.as_ref() == id);
    requested_by_policy || (state.open_before && !should_close)
}

pub(in crate::imui::menu_family_controls) fn activate_menubar_trigger_if_requested<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    open_requested: bool,
    state: &BeginMenuState,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(menubar_policy) = state.menubar_policy.as_ref() else {
        return;
    };
    if !open_requested {
        return;
    }
    let Some(trigger_id) = trigger_id else {
        return;
    };

    ui.with_cx_mut(|cx| {
        let open_for_state = state.row_open.clone();
        let _ = cx
            .app
            .models_mut()
            .update(&menubar_policy.group_active, |value| {
                *value = Some(menubar_trigger_row::MenubarActiveTrigger {
                    trigger: trigger_id,
                    open: open_for_state,
                });
            });
        let _ = cx
            .app
            .models_mut()
            .update(&state.row_open, |value| *value = true);
    });
}

pub(in crate::imui::menu_family_controls) fn close_disabled_popup_if_opened<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    enabled: bool,
    popup_opened: bool,
) {
    if enabled || !popup_opened {
        return;
    }

    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&state.row_open, |value| *value = false);
    });
    ui.close_popup(id);
}
