use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) fn install_radio_activation<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
        crate::imui::mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model, false);
        host.record_transient_event(acx, crate::imui::KEY_CLICKED);
        host.notify(acx);
    }));
}
