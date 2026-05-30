use fret_runtime::{KeyChord, Model};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ResponseExt, active_trigger_behavior};

mod activation;
mod keyboard;
mod response;

pub(super) struct SwitchBehaviorOptions {
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_switch_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    model: Model<bool>,
    options: SwitchBehaviorOptions,
    response: &mut ResponseExt,
) {
    let SwitchBehaviorOptions {
        enabled,
        focusable,
        activate_shortcut,
        shortcut_repeat,
    } = options;
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions {
            primary_active: true,
            request_focus_on_press: false,
            clear_pointer_move: true,
        },
    );

    activation::install_switch_activation(
        cx,
        activation::SwitchActivationInput {
            model: model.clone(),
            lifecycle_model: behavior.lifecycle_model.clone(),
        },
    );

    if enabled && focusable {
        keyboard::install_switch_keyboard(
            cx,
            id,
            keyboard::SwitchKeyboardInput {
                model,
                activate_shortcut,
                shortcut_repeat,
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );
    }

    response::populate_switch_response(cx, id, state, &behavior, enabled, response);
}
