use std::sync::Arc;

use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;

use super::super::capture::BeginMenuState;

mod activate;
mod read;
mod reconcile;

use activate::activate_requested_trigger;
use read::trigger_is_active;

pub(in crate::imui::menu_family_controls) use reconcile::reconcile_menubar_after_trigger;

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

    let is_active_trigger = trigger_is_active(ui, policy, trigger_id);
    if is_active_trigger {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&policy.open_menu, |value| *value = Some(Arc::from(id)));
        });
    }
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
    activate_requested_trigger(ui, menubar_policy, &state.row_open, trigger_id);
}
