use fret_runtime::Model;
use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

pub(in crate::imui::menu_family_controls) fn activate_requested_trigger<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    menubar_policy: &ImUiMenubarPolicyState,
    row_open: &Model<bool>,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(trigger_id) = trigger_id else {
        return;
    };

    ui.with_cx_mut(|cx| {
        let open_for_state = row_open.clone();
        let _ = cx
            .app
            .models_mut()
            .update(&menubar_policy.group_active, |value| {
                *value = Some(menubar_trigger_row::MenubarActiveTrigger {
                    trigger: trigger_id,
                    open: open_for_state,
                });
            });
        let _ = cx.app.models_mut().update(row_open, |value| *value = true);
    });
}
