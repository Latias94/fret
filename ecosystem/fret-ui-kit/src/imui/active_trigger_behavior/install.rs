use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ActiveTriggerBehavior, ActiveTriggerBehaviorOptions, keyboard, pointer};

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

    let active_item_model = super::super::active_item_model_for_window(cx);
    let lifecycle_model = super::super::lifecycle_session_model_for(cx, id);
    let context_anchor_model = super::super::context_menu_anchor_model_for(cx, id);

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
