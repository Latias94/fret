use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::interaction_runtime::ImUiLifecycleSessionState;

mod context;
mod shortcut;

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
            shortcut::handle_combo_trigger_activate_shortcut(
                host,
                acx,
                down,
                input.activate_shortcut,
                input.shortcut_repeat,
                &lifecycle_model_for_shortcut,
            ) || context::handle_combo_trigger_context_menu_key(host, acx, down)
        }),
    );
}
