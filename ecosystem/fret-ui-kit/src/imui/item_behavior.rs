//! Private shared item behavior for immediate-mode pressable controls.

use fret_core::{Modifiers, MouseButton, Point};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::interaction_runtime::{
    ImUiActiveItemState, ImUiLifecycleSessionState, LongPressSignalState,
};

mod response;

pub(super) use response::populate_pressable_item_response;

pub(super) struct PressableItemBehavior {
    pub(super) active_item_model: Model<ImUiActiveItemState>,
    pub(super) context_anchor_model: Model<Option<Point>>,
    pub(super) long_press_signal_model: Model<LongPressSignalState>,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
    pointer_click_modifiers_model: Option<Model<Modifiers>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PressableItemBehaviorOptions {
    pub(super) report_pointer_click: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PressableItemResponseInput {
    pub(super) enabled: bool,
    pub(super) clicked: bool,
    pub(super) changed: bool,
    pub(super) lifecycle_edited: bool,
}

pub(super) fn install_pressable_item_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> PressableItemBehavior {
    install_pressable_item_behavior_with_options(cx, id, PressableItemBehaviorOptions::default())
}

pub(super) fn install_pressable_item_behavior_with_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    options: PressableItemBehaviorOptions,
) -> PressableItemBehavior {
    cx.pressable_clear_on_pointer_down();
    cx.pressable_clear_on_pointer_move();
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::active_item_model_for_window(cx);
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_move = active_item_model.clone();
    let active_item_model_for_up = active_item_model.clone();

    let context_anchor_model = super::context_menu_anchor_model_for(cx, id);
    let context_anchor_model_for_up = context_anchor_model.clone();

    let long_press_signal_model = super::long_press_signal_model_for(cx, id);
    let long_press_signal_model_for_down = long_press_signal_model.clone();
    let long_press_signal_model_for_move = long_press_signal_model.clone();
    let long_press_signal_model_for_up = long_press_signal_model.clone();

    let lifecycle_model = super::lifecycle_session_model_for(cx, id);
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model.clone();

    let pointer_click_modifiers_model = options
        .report_pointer_click
        .then(|| super::pointer_click_modifiers_model_for(cx, id));
    let pointer_click_modifiers_model_for_up = pointer_click_modifiers_model.clone();

    cx.pressable_on_pointer_down(std::sync::Arc::new(move |host, acx, down| {
        super::mark_lifecycle_activated_on_left_pointer_down(
            host,
            acx,
            down.button,
            &lifecycle_model_for_down,
        );
        super::prepare_pressable_drag_on_pointer_down(
            host,
            acx,
            down,
            &active_item_model_for_down,
            &long_press_signal_model_for_down,
            super::drag_kind_for_element(acx.target),
        );

        PressablePointerDownResult::Continue
    }));

    let drag_threshold = super::drag_threshold_for(cx);
    cx.pressable_on_pointer_move(std::sync::Arc::new(move |host, acx, mv| {
        super::handle_pressable_drag_move_with_threshold(
            host,
            acx,
            mv,
            &active_item_model_for_move,
            &long_press_signal_model_for_move,
            super::drag_kind_for_element(acx.target),
            drag_threshold,
        )
    }));

    cx.pressable_on_pointer_up(std::sync::Arc::new(move |host, acx, up| {
        super::mark_lifecycle_deactivated_on_left_pointer_up(
            host,
            acx,
            up.button,
            &lifecycle_model_for_up,
        );
        super::finish_pressable_drag_on_pointer_up(
            host,
            acx,
            up,
            &active_item_model_for_up,
            &long_press_signal_model_for_up,
            super::drag_kind_for_element(acx.target),
        );

        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model_for_up, |v| *v = Some(up.position));
            host.record_transient_event(acx, super::KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
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
            host.record_transient_event(acx, super::KEY_POINTER_CLICKED);
        }

        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
            host.record_transient_event(acx, super::KEY_DOUBLE_CLICKED);
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
