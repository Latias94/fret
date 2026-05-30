use fret_runtime::{ActionId, KeyChord, Model};
use fret_ui::action::{ActivateReason, KeyDownCx, UiActionHostExt as _};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;
use crate::imui::{KEY_CLICKED, active_trigger_behavior, mark_lifecycle_instant_if_inactive};

use super::super::super::interaction::MenuItemInteraction;

pub(super) struct PopupMenuShortcut {
    close_popup: Option<Model<bool>>,
    action: Option<ActionId>,
    activate_shortcut: Option<KeyChord>,
    shortcut_repeat: bool,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
}

pub(super) fn popup_menu_shortcut(
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    interaction: &MenuItemInteraction,
) -> PopupMenuShortcut {
    PopupMenuShortcut {
        close_popup: interaction.close_popup.clone(),
        action: interaction.action.clone(),
        activate_shortcut: interaction.activate_shortcut,
        shortcut_repeat: interaction.shortcut_repeat,
        lifecycle_model: behavior.lifecycle_model.clone(),
    }
}

pub(super) fn handle_popup_menu_shortcut(
    host: &mut dyn fret_ui::action::UiActionHost,
    acx: fret_ui::action::ActionCx,
    down: &KeyDownCx,
    shortcut: &PopupMenuShortcut,
) -> bool {
    let Some(chord) = shortcut.activate_shortcut else {
        return false;
    };
    let matches_shortcut = down.key == chord.key && down.modifiers == chord.mods;
    if !matches_shortcut || (down.repeat && !shortcut.shortcut_repeat) || down.ime_composing {
        return false;
    }

    mark_lifecycle_instant_if_inactive(host, acx, &shortcut.lifecycle_model, false);
    if let Some(open) = shortcut.close_popup.as_ref() {
        let _ = host.update_model(open, |v| *v = false);
    }
    host.record_transient_event(acx, KEY_CLICKED);
    super::super::super::interaction::dispatch_menu_item_action(
        host,
        acx,
        ActivateReason::Keyboard,
        shortcut.action.clone(),
    );
    host.notify(acx);
    true
}
