use fret_runtime::Model;
use fret_ui::action::ActivateReason;
use fret_ui::{ElementContext, UiHost};

use super::action::{ButtonAction, dispatch_button_action};
use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct ButtonActivationBehaviorInput {
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
    pub(super) action: Option<ButtonAction>,
}

pub(super) fn install_button_activation_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: ButtonActivationBehaviorInput,
) {
    let lifecycle_model_for_activate = input.lifecycle_model.clone();
    let action_for_activate = input.action.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            crate::imui::mark_lifecycle_instant_if_inactive(
                host,
                acx,
                &lifecycle_model_for_activate,
                false,
            );
        }
        host.record_transient_event(acx, crate::imui::KEY_CLICKED);
        dispatch_button_action(host, acx, reason, action_for_activate.clone());
        host.notify(acx);
    }));
}
