use fret_runtime::ActionId;
use fret_ui::action::{ActivateReason, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use crate::imui::{KEY_CLICKED, active_trigger_behavior, mark_lifecycle_instant_if_inactive};

use super::super::MenuItemInteraction;

pub(super) fn install_menu_item_activate_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    interaction: &MenuItemInteraction,
) {
    let close_popup_for_activate = interaction.close_popup.clone();
    let action_for_activate = interaction.action.clone();
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
        }
        if let Some(open) = close_popup_for_activate.as_ref() {
            let _ = host.update_model(open, |v| *v = false);
        }
        host.record_transient_event(acx, KEY_CLICKED);
        dispatch_menu_item_action(host, acx, reason, action_for_activate.clone());
        host.notify(acx);
    }));
}

pub(in crate::imui::menu_controls::interaction) fn dispatch_menu_item_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    acx: fret_ui::action::ActionCx,
    reason: ActivateReason,
    action: Option<ActionId>,
) {
    if let Some(action) = action {
        host.record_pending_command_dispatch_source(acx, &action, reason);
        host.dispatch_command(Some(acx.window), action);
    }
}
