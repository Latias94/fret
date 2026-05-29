use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::imui::popup_overlay::ImUiPopupMenuPolicyState;

mod reset;

use reset::{
    clear_active_submenu_models, clear_pending_submenu_models, clear_submenu_runtime_models,
};

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
        let models = &popup_policy.submenu_models;
        clear_active_submenu_models(cx, models, submenu_value, trigger_id, clear_geometry);
        clear_pending_submenu_models(cx, models, submenu_value, trigger_id);
        clear_submenu_runtime_models(cx, models);
    });
}
