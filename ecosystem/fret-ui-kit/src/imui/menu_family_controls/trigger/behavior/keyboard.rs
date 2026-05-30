use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct MenuTriggerKeyboardInput {
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

pub(super) fn install_menu_trigger_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    input: MenuTriggerKeyboardInput,
) {
    let lifecycle_model_for_shortcut = input.lifecycle_model.clone();
    let activate_shortcut = input.activate_shortcut;
    let shortcut_repeat = input.shortcut_repeat;
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut && (!down.repeat || shortcut_repeat) && !down.ime_composing {
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

            false
        }),
    );
}
