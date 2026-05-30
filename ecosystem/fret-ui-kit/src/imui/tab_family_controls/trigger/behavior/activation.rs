use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActivateReason, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct TabTriggerActivationInput {
    pub(super) selected_model: Model<Option<Arc<str>>>,
    pub(super) tab_id: Arc<str>,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

pub(super) fn install_tab_trigger_activation<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: TabTriggerActivationInput,
) {
    let selected_model_for_activate = input.selected_model.clone();
    let tab_id_for_activate = input.tab_id.clone();
    let lifecycle_model_for_activate = input.lifecycle_model.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            crate::imui::mark_lifecycle_instant_if_inactive(
                host,
                acx,
                &lifecycle_model_for_activate,
                false,
            );
        }
        let _ = host.update_model(&selected_model_for_activate, |value| {
            *value = Some(tab_id_for_activate.clone())
        });
        host.record_transient_event(acx, crate::imui::KEY_CLICKED);
        host.notify(acx);
    }));
}
