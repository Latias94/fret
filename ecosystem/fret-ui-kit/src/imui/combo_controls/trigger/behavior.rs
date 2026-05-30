use fret_runtime::KeyChord;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, item_behavior};

mod activation;
mod keyboard;
mod response;

pub(super) struct ComboTriggerBehaviorInput {
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_combo_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: ComboTriggerBehaviorInput,
    response: &mut ResponseExt,
) {
    let ComboTriggerBehaviorInput {
        enabled,
        activate_shortcut,
        shortcut_repeat,
    } = input;
    let behavior = item_behavior::install_pressable_item_behavior(cx, id);

    activation::install_combo_trigger_activation(cx, behavior.lifecycle_model.clone());

    if enabled {
        keyboard::install_combo_trigger_keyboard(
            cx,
            id,
            keyboard::ComboTriggerKeyboardInput {
                activate_shortcut,
                shortcut_repeat,
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );
    }

    response::populate_combo_trigger_response(cx, id, state, &behavior, enabled, response);
}
