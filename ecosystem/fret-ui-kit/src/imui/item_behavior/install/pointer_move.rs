use std::sync::Arc;

use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::{ImUiActiveItemState, LongPressSignalState};

pub(super) fn install_pointer_move<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: fret_runtime::Model<ImUiActiveItemState>,
    long_press_signal_model: fret_runtime::Model<LongPressSignalState>,
) {
    let drag_threshold = crate::imui::drag_threshold_for(cx);
    cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
        crate::imui::handle_pressable_drag_move_with_threshold(
            host,
            acx,
            mv,
            &active_item_model,
            &long_press_signal_model,
            crate::imui::drag_kind_for_element(acx.target),
            drag_threshold,
        )
    }));
}
