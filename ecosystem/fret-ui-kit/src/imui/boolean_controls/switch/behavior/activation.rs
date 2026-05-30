use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct SwitchActivationInput {
    pub(super) model: Model<bool>,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

pub(super) fn install_switch_activation<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: SwitchActivationInput,
) {
    let model_for_activate = input.model.clone();
    let lifecycle_model_for_activate = input.lifecycle_model.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
        let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
        crate::imui::mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
        host.record_transient_event(acx, crate::imui::KEY_CLICKED);
        host.record_transient_event(acx, crate::imui::KEY_CHANGED);
        host.notify(acx);
    }));
}
