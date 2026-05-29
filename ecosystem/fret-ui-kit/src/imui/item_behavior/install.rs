use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{PressableItemBehavior, PressableItemBehaviorOptions};

mod pointer_down;
mod pointer_move;
mod pointer_up;

pub(in crate::imui) fn install_pressable_item_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> PressableItemBehavior {
    install_pressable_item_behavior_with_options(cx, id, PressableItemBehaviorOptions::default())
}

pub(in crate::imui) fn install_pressable_item_behavior_with_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    options: PressableItemBehaviorOptions,
) -> PressableItemBehavior {
    cx.pressable_clear_on_pointer_down();
    cx.pressable_clear_on_pointer_move();
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::super::active_item_model_for_window(cx);
    let context_anchor_model = super::super::context_menu_anchor_model_for(cx, id);
    let long_press_signal_model = super::super::long_press_signal_model_for(cx, id);
    let lifecycle_model = super::super::lifecycle_session_model_for(cx, id);
    let pointer_click_modifiers_model = options
        .report_pointer_click
        .then(|| super::super::pointer_click_modifiers_model_for(cx, id));

    pointer_down::install_pointer_down(
        cx,
        active_item_model.clone(),
        long_press_signal_model.clone(),
        lifecycle_model.clone(),
    );
    pointer_move::install_pointer_move(
        cx,
        active_item_model.clone(),
        long_press_signal_model.clone(),
    );
    pointer_up::install_pointer_up(
        cx,
        active_item_model.clone(),
        context_anchor_model.clone(),
        long_press_signal_model.clone(),
        lifecycle_model.clone(),
        pointer_click_modifiers_model.clone(),
    );

    PressableItemBehavior {
        active_item_model,
        context_anchor_model,
        long_press_signal_model,
        lifecycle_model,
        pointer_click_modifiers_model,
    }
}
