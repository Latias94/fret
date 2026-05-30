use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::{KeyChord, Model};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct ComboTriggerKeyboardInput {
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

pub(super) fn install_combo_trigger_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    input: ComboTriggerKeyboardInput,
) {
    let lifecycle_model_for_shortcut = input.lifecycle_model.clone();
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = input.activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut
                    && (!down.repeat || input.shortcut_repeat)
                    && !down.ime_composing
                {
                    crate::imui::mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_shortcut,
                        false,
                    );
                    host.record_transient_event(acx, crate::imui::KEY_CLICKED);
                    host.notify(acx);
                    return true;
                }
            }

            let is_menu_key = down.key == KeyCode::ContextMenu;
            let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
            if !(is_menu_key || is_shift_f10) {
                return false;
            }

            host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            true
        }),
    );
}
