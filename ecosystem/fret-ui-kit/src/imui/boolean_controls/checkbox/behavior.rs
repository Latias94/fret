use fret_runtime::{KeyChord, Model};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ResponseExt, item_behavior};

mod activation;
mod keyboard;
mod response;

pub(super) struct CheckboxBehaviorOptions {
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_checkbox_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    model: Model<bool>,
    options: CheckboxBehaviorOptions,
    response: &mut ResponseExt,
) {
    let CheckboxBehaviorOptions {
        enabled,
        activate_shortcut,
        shortcut_repeat,
    } = options;
    let behavior = item_behavior::install_pressable_item_behavior(cx, id);

    activation::install_checkbox_activation(
        cx,
        activation::CheckboxActivationInput {
            model: model.clone(),
            lifecycle_model: behavior.lifecycle_model.clone(),
        },
    );

    if enabled {
        keyboard::install_checkbox_keyboard(
            cx,
            id,
            keyboard::CheckboxKeyboardInput {
                model,
                activate_shortcut,
                shortcut_repeat,
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );
    }

    response::populate_checkbox_response(cx, id, state, &behavior, enabled, response);
}
