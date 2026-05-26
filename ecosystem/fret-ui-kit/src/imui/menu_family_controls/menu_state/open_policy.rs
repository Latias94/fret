use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

use super::capture::{BeginMenuState, read_bool_model, read_open_menu_model};

mod active_trigger;

pub(in crate::imui::menu_family_controls) use active_trigger::{
    activate_menubar_trigger_if_requested, reconcile_menubar_after_trigger,
    sync_open_menu_for_active_trigger,
};

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
