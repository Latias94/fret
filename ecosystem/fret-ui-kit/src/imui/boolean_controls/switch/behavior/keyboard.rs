use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct SwitchKeyboardInput {
    pub(super) model: Model<bool>,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

pub(super) fn install_switch_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    input: SwitchKeyboardInput,
) {
    let model_for_shortcut = input.model.clone();
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
                    let _ = host.update_model(&model_for_shortcut, |v: &mut bool| *v = !*v);
                    crate::imui::mark_lifecycle_edit(host, acx, &lifecycle_model_for_shortcut);
                    host.record_transient_event(acx, crate::imui::KEY_CLICKED);
                    host.record_transient_event(acx, crate::imui::KEY_CHANGED);
                    host.notify(acx);
                    return true;
                }
            }

            false
        }),
    );
}
