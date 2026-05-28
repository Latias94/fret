use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;

use super::super::super::capture::BeginMenuState;

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
