use fret_runtime::ActionId;
use fret_ui::action::{ActionCx, ActivateReason, UiActionHost};

/// Dispatch a unit action through the existing runtime command pipeline.
pub(crate) fn dispatch_action(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    reason: ActivateReason,
    action: &ActionId,
) {
    host.record_pending_command_dispatch_source(action_cx, action, reason);
    host.dispatch_command(Some(action_cx.window), action.clone());
}
