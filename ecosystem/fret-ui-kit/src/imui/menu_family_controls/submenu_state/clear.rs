use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::imui::popup_overlay::ImUiPopupMenuPolicyState;

pub(in crate::imui::menu_family_controls) fn clear_imui_submenu<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    popup_policy: &ImUiPopupMenuPolicyState,
    submenu_value: &str,
    trigger_id: Option<GlobalElementId>,
    clear_geometry: bool,
) {
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_value, |value| {
                if value
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == submenu_value)
                {
                    *value = None;
                }
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.trigger, |value| {
                if *value == trigger_id {
                    *value = None;
                }
            });
        if clear_geometry {
            let _ = cx
                .app
                .models_mut()
                .update(&popup_policy.submenu_models.geometry, |value| *value = None);
        }
        let _ =
            cx.app
                .models_mut()
                .update(&popup_policy.submenu_models.pending_open_value, |value| {
                    if value
                        .as_ref()
                        .is_some_and(|current| current.as_ref() == submenu_value)
                    {
                        *value = None;
                    }
                });
        let _ = cx.app.models_mut().update(
            &popup_policy.submenu_models.pending_open_trigger,
            |value| {
                if *value == trigger_id {
                    *value = None;
                }
            },
        );
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pointer_grace_intent, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pointer_grace_timer, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.close_timer, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.focus_target, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.focus_timer, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.focus_retry_attempts, |value| {
                *value = 0
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_timer, |value| {
                *value = None
            });
    });
}
