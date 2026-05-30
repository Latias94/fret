use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, active_trigger_behavior};

mod activation;
mod keyboard;
mod response;

pub(super) struct TabTriggerBehaviorInput {
    pub(super) selected_model: Model<Option<Arc<str>>>,
    pub(super) tab_id: Arc<str>,
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_tab_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: TabTriggerBehaviorInput,
    response: &mut ResponseExt,
) {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
    );

    if input.enabled {
        activation::install_tab_trigger_activation(
            cx,
            activation::TabTriggerActivationInput {
                selected_model: input.selected_model.clone(),
                tab_id: input.tab_id.clone(),
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );

        keyboard::install_tab_trigger_keyboard(
            cx,
            id,
            keyboard::TabTriggerKeyboardInput {
                selected_model: input.selected_model.clone(),
                tab_id: input.tab_id.clone(),
                activate_shortcut: input.activate_shortcut,
                shortcut_repeat: input.shortcut_repeat,
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );
    }

    response::populate_tab_trigger_response(cx, id, state, &behavior, input.enabled, response);
}
