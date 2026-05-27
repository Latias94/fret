use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use super::ActiveTriggerBehaviorOptions;
use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};
use crate::imui::{
    KEY_CONTEXT_MENU_REQUESTED, KEY_SECONDARY_CLICKED, clear_active_item_on_left_pointer_up,
    mark_active_item_on_left_pointer_down, mark_lifecycle_activated_on_left_pointer_down,
    mark_lifecycle_deactivated_on_left_pointer_up,
};

pub(super) fn install_active_trigger_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
    context_anchor_model: Model<Option<fret_core::Point>>,
    options: ActiveTriggerBehaviorOptions,
) {
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_up = active_item_model;
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model;
    let primary_active = options.primary_active;
    let request_focus_on_press = options.request_focus_on_press;

    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        if primary_active {
            mark_lifecycle_activated_on_left_pointer_down(
                host,
                acx,
                down.button,
                &lifecycle_model_for_down,
            );
            mark_active_item_on_left_pointer_down(
                host,
                acx,
                down.button,
                &active_item_model_for_down,
                request_focus_on_press,
            );
        }
        PressablePointerDownResult::Continue
    }));

    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if primary_active {
            mark_lifecycle_deactivated_on_left_pointer_up(
                host,
                acx,
                up.button,
                &lifecycle_model_for_up,
            );
            clear_active_item_on_left_pointer_up(host, acx, up.button, &active_item_model_for_up);
        }
        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model, |value| *value = Some(up.position));
            host.record_transient_event(acx, KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            return PressablePointerUpResult::SkipActivate;
        }
        PressablePointerUpResult::Continue
    }));
}
