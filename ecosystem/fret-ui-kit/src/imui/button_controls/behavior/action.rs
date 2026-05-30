use std::{any::Any, sync::Arc};

use fret_runtime::ActionId;
use fret_ui::action::{ActionCx, ActivateReason, UiActionHost};

#[derive(Clone)]
pub(in crate::imui::button_controls) struct ButtonAction {
    pub(in crate::imui::button_controls) action: ActionId,
    pub(in crate::imui::button_controls) payload:
        Option<Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>>,
}

pub(super) fn dispatch_button_action(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    reason: ActivateReason,
    action: Option<ButtonAction>,
) {
    if let Some(action) = action {
        host.record_pending_command_dispatch_source(acx, &action.action, reason);
        if let Some(payload) = action.payload.as_ref() {
            host.record_pending_action_payload(acx, &action.action, payload());
        }
        host.dispatch_command(Some(acx.window), action.action);
    }
}
