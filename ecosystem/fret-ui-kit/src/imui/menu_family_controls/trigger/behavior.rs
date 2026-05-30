use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, active_trigger_behavior};

use super::super::ImUiMenubarPolicyState;

mod activation;
mod keyboard;
mod menubar;
mod response;

pub(super) struct MenuTriggerBehaviorInput {
    pub(super) logical_key: Arc<str>,
    pub(super) open_model: Model<bool>,
    pub(super) menubar_policy: Option<ImUiMenubarPolicyState>,
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_menu_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: MenuTriggerBehaviorInput,
    response: &mut ResponseExt,
) {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
    );

    activation::install_menu_trigger_activation(cx, behavior.lifecycle_model.clone());

    if input.enabled {
        keyboard::install_menu_trigger_keyboard(
            cx,
            id,
            keyboard::MenuTriggerKeyboardInput {
                activate_shortcut: input.activate_shortcut,
                shortcut_repeat: input.shortcut_repeat,
                lifecycle_model: behavior.lifecycle_model.clone(),
            },
        );
    }

    menubar::install_menubar_trigger_behavior(cx, id, state, &input);

    response::populate_menu_trigger_response(cx, id, state, &behavior, input.enabled, response);
}
