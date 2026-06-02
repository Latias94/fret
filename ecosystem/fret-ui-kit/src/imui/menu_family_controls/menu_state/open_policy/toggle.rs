use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

use super::super::capture::BeginMenuState;

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
