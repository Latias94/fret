use std::sync::Arc;

use fret_ui::action::PressablePointerDownResult;
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::{
    ImUiActiveItemState, ImUiLifecycleSessionState, LongPressSignalState,
};

pub(super) fn install_pointer_down<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: fret_runtime::Model<ImUiActiveItemState>,
    long_press_signal_model: fret_runtime::Model<LongPressSignalState>,
    lifecycle_model: fret_runtime::Model<ImUiLifecycleSessionState>,
) {
    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        crate::imui::mark_lifecycle_activated_on_left_pointer_down(
            host,
            acx,
            down.button,
            &lifecycle_model,
        );
        crate::imui::prepare_pressable_drag_on_pointer_down(
            host,
            acx,
            down,
            &active_item_model,
            &long_press_signal_model,
            crate::imui::drag_kind_for_element(acx.target),
        );

        PressablePointerDownResult::Continue
    }));
}
