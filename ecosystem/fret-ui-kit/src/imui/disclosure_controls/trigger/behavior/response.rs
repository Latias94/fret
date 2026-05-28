use fret_runtime::Model;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use crate::imui::{
    KEY_CLICKED, KEY_CONTEXT_MENU_REQUESTED, KEY_DOUBLE_CLICKED, KEY_SECONDARY_CLICKED,
    ResponseExt, active_item_model_for_window, hover_blocked_by_active_item_for,
    install_hover_query_hooks_for_pressable, sanitize_response_for_enabled,
};

pub(super) fn populate_disclosure_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    state: &PressableState,
    context_anchor_model: Model<Option<fret_core::Point>>,
    enabled: bool,
    trigger_response: &mut ResponseExt,
) {
    let active_item_model = active_item_model_for_window(cx);
    trigger_response.set_core_hovered(state.hovered);
    trigger_response.set_core_pressed(state.pressed);
    trigger_response.set_core_focused(state.focused);
    trigger_response.set_nav_highlighted(
        state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
    );
    trigger_response.set_id(Some(trigger_id));
    trigger_response.set_core_clicked(cx.take_transient_for(trigger_id, KEY_CLICKED));
    trigger_response
        .set_secondary_clicked(cx.take_transient_for(trigger_id, KEY_SECONDARY_CLICKED));
    trigger_response.set_double_clicked(cx.take_transient_for(trigger_id, KEY_DOUBLE_CLICKED));
    trigger_response
        .set_context_menu_requested(cx.take_transient_for(trigger_id, KEY_CONTEXT_MENU_REQUESTED));
    trigger_response.set_context_menu_anchor(
        cx.read_model(&context_anchor_model, Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(None),
    );
    trigger_response.set_core_rect(cx.last_bounds_for_element(trigger_id));
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
    sanitize_response_for_enabled(enabled, trigger_response);
}
