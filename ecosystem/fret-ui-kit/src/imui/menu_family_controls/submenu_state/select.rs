use std::sync::Arc;

use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::imui::popup_overlay::ImUiPopupMenuPolicyState;

pub(in crate::imui::menu_family_controls) fn select_imui_submenu<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    popup_policy: &ImUiPopupMenuPolicyState,
    submenu_value: Arc<str>,
    trigger_id: Option<GlobalElementId>,
) {
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_value, |value| {
                *value = Some(submenu_value.clone());
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.trigger, |value| {
                *value = trigger_id
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pending_open_value, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pending_open_trigger, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_timer, |value| {
                *value = None
            });
    });
}
