use fret_runtime::{KeyChord, Model};
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) fn handle_combo_trigger_activate_shortcut(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    down: KeyDownCx,
    activate_shortcut: Option<KeyChord>,
    shortcut_repeat: bool,
    lifecycle_model: &Model<ImUiLifecycleSessionState>,
) -> bool {
    let Some(shortcut) = activate_shortcut else {
        return false;
    };

    let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
    if !matches_shortcut || (down.repeat && !shortcut_repeat) || down.ime_composing {
        return false;
    }

    crate::imui::mark_lifecycle_instant_if_inactive(host, acx, lifecycle_model, false);
    host.record_transient_event(acx, crate::imui::KEY_CLICKED);
    host.notify(acx);
    true
}
