use fret_core::KeyCode;
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};

pub(super) fn handle_combo_trigger_context_menu_key(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    down: KeyDownCx,
) -> bool {
    let is_menu_key = down.key == KeyCode::ContextMenu;
    let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
    if !(is_menu_key || is_shift_f10) {
        return false;
    }

    host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
    host.notify(acx);
    true
}
