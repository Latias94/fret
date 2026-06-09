use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::PressablePointerDownResult;
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

pub(super) fn install_active_trigger_pointer_down<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
    primary_active: bool,
    request_focus_on_press: bool,
) {
    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        if primary_active {
            crate::imui::mark_lifecycle_activated_on_left_pointer_down(
                host,
                acx,
                down.button,
                &lifecycle_model,
            );
            crate::imui::mark_active_item_on_left_pointer_down(
                host,
                acx,
                down.button,
                &active_item_model,
                request_focus_on_press,
            );
        }

        PressablePointerDownResult::Continue
    }));
}
