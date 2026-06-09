use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ResponseExt, item_behavior};

mod activation;
mod keyboard;
mod options;
mod response;

pub(super) use options::RadioBehaviorOptions;

pub(super) fn install_radio_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    options: RadioBehaviorOptions,
    response: &mut ResponseExt,
) {
    let RadioBehaviorOptions {
        enabled,
        activate_shortcut,
        shortcut_repeat,
    } = options;
    let behavior = item_behavior::install_pressable_item_behavior(cx, id);

    activation::install_radio_activation(cx, behavior.lifecycle_model.clone());

    if enabled {
        keyboard::install_radio_keyboard(
            cx,
            id,
            keyboard::RadioKeyboardInput {
                activate_shortcut,
                shortcut_repeat,
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );
    }

    response::populate_radio_response(cx, id, state, &behavior, enabled, response);
}
