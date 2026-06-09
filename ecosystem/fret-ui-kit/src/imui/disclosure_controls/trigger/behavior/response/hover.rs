use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    ResponseExt, active_item_model_for_window, hover_blocked_by_active_item_for,
    install_hover_query_hooks_for_pressable,
};

pub(super) fn populate_disclosure_trigger_hover_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    state: &PressableState,
    trigger_response: &mut ResponseExt,
) {
    let active_item_model = active_item_model_for_window(cx);
    let hover_delay =
        install_hover_query_hooks_for_pressable(cx, trigger_id, state.hovered_raw, None);
    trigger_response.set_pointer_hovered_raw(state.hovered_raw);
    trigger_response.set_pointer_hovered_raw_below_barrier(state.hovered_raw_below_barrier);
    trigger_response.set_hover_stationary_met(hover_delay.stationary_met);
    trigger_response.set_hover_delay_short_met(hover_delay.delay_short_met);
    trigger_response.set_hover_delay_normal_met(hover_delay.delay_normal_met);
    trigger_response.set_hover_delay_short_shared_met(hover_delay.shared_delay_short_met);
    trigger_response.set_hover_delay_normal_shared_met(hover_delay.shared_delay_normal_met);
    trigger_response.set_hover_blocked_by_active_item(hover_blocked_by_active_item_for(
        cx,
        trigger_id,
        &active_item_model,
    ));
}
