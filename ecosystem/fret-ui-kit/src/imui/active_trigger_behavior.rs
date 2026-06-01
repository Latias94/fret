//! Private shared behavior for active-only immediate-mode triggers.

use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod keyboard;
mod pointer;
mod response;
mod types;

pub(super) use types::{
    ActiveTriggerBehavior, ActiveTriggerBehaviorOptions, ActiveTriggerResponseInput,
};

pub(super) fn install_active_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    options: ActiveTriggerBehaviorOptions,
) -> ActiveTriggerBehavior {
    cx.pressable_clear_on_pointer_down();
    if options.clear_pointer_move {
        cx.pressable_clear_on_pointer_move();
    }
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::active_item_model_for_window(cx);
    let lifecycle_model = super::lifecycle_session_model_for(cx, id);
    let context_anchor_model = super::context_menu_anchor_model_for(cx, id);

    keyboard::install_context_menu_key_handler(cx, id);
    pointer::install_active_trigger_pointer_handlers(
        cx,
        active_item_model.clone(),
        lifecycle_model.clone(),
        context_anchor_model.clone(),
        options,
    );

    ActiveTriggerBehavior {
        active_item_model,
        context_anchor_model,
        lifecycle_model,
    }
}

pub(super) fn populate_active_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &ActiveTriggerBehavior,
    input: ActiveTriggerResponseInput,
    response: &mut super::ResponseExt,
) {
    response::populate_active_trigger_response(cx, id, state, behavior, input, response);
}
