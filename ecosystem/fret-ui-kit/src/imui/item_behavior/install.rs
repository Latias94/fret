use std::sync::Arc;

use fret_core::MouseButton;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{PressableItemBehavior, PressableItemBehaviorOptions};

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
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_move = active_item_model.clone();
    let active_item_model_for_up = active_item_model.clone();

    let context_anchor_model = super::super::context_menu_anchor_model_for(cx, id);
    let context_anchor_model_for_up = context_anchor_model.clone();

    let long_press_signal_model = super::super::long_press_signal_model_for(cx, id);
    let long_press_signal_model_for_down = long_press_signal_model.clone();
    let long_press_signal_model_for_move = long_press_signal_model.clone();
    let long_press_signal_model_for_up = long_press_signal_model.clone();

    let lifecycle_model = super::super::lifecycle_session_model_for(cx, id);
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model.clone();

    let pointer_click_modifiers_model = options
        .report_pointer_click
        .then(|| super::super::pointer_click_modifiers_model_for(cx, id));
    let pointer_click_modifiers_model_for_up = pointer_click_modifiers_model.clone();

    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        super::super::mark_lifecycle_activated_on_left_pointer_down(
            host,
            acx,
            down.button,
            &lifecycle_model_for_down,
        );
        super::super::prepare_pressable_drag_on_pointer_down(
            host,
            acx,
            down,
            &active_item_model_for_down,
            &long_press_signal_model_for_down,
            super::super::drag_kind_for_element(acx.target),
        );

        PressablePointerDownResult::Continue
    }));

    let drag_threshold = super::super::drag_threshold_for(cx);
    cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
        super::super::handle_pressable_drag_move_with_threshold(
            host,
            acx,
            mv,
            &active_item_model_for_move,
            &long_press_signal_model_for_move,
            super::super::drag_kind_for_element(acx.target),
            drag_threshold,
        )
    }));

    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        super::super::mark_lifecycle_deactivated_on_left_pointer_up(
            host,
            acx,
            up.button,
            &lifecycle_model_for_up,
        );
        super::super::finish_pressable_drag_on_pointer_up(
            host,
            acx,
            up,
            &active_item_model_for_up,
            &long_press_signal_model_for_up,
            super::super::drag_kind_for_element(acx.target),
        );

        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model_for_up, |v| *v = Some(up.position));
            host.record_transient_event(acx, super::super::KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, super::super::KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            return PressablePointerUpResult::SkipActivate;
        }

        if up.is_click
            && up.button == MouseButton::Left
            && let Some(pointer_click_modifiers_model) =
                pointer_click_modifiers_model_for_up.as_ref()
        {
            let _ = host.update_model(pointer_click_modifiers_model, |value| {
                *value = up.modifiers;
            });
            host.record_transient_event(acx, super::super::KEY_POINTER_CLICKED);
        }

        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
            host.record_transient_event(acx, super::super::KEY_DOUBLE_CLICKED);
            host.notify(acx);
        }

        PressablePointerUpResult::Continue
    }));

    PressableItemBehavior {
        active_item_model,
        context_anchor_model,
        long_press_signal_model,
        lifecycle_model,
        pointer_click_modifiers_model,
    }
}
