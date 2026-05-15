use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{HoverQueryDelayRead, ImUiActiveItemState};

pub(in super::super) fn populate_pressable_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    hover_delay: HoverQueryDelayRead,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
    clicked: bool,
    changed: bool,
    active_now: bool,
    lifecycle_edited: bool,
    enabled: bool,
    response: &mut super::super::ResponseExt,
) {
    response.set_core_hovered(state.hovered);
    response.set_core_pressed(state.pressed);
    response.set_core_focused(state.focused);
    response.set_nav_highlighted(
        state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
    );
    response.set_id(Some(id));
    response.set_core_clicked(clicked);
    response.set_core_changed(changed);
    response.set_core_rect(cx.last_bounds_for_element(id));
    response.set_pointer_hovered_raw(state.hovered_raw);
    response.set_pointer_hovered_raw_below_barrier(state.hovered_raw_below_barrier);
    response.set_hover_stationary_met(hover_delay.stationary_met);
    response.set_hover_delay_short_met(hover_delay.delay_short_met);
    response.set_hover_delay_normal_met(hover_delay.delay_normal_met);
    response.set_hover_delay_short_shared_met(hover_delay.shared_delay_short_met);
    response.set_hover_delay_normal_shared_met(hover_delay.shared_delay_normal_met);
    response.set_hover_blocked_by_active_item(super::hover_blocked_by_active_item_for(
        cx,
        id,
        active_item_model,
    ));
    super::populate_response_lifecycle_transients(cx, id, response);
    super::populate_response_lifecycle_from_active_state(
        cx,
        id,
        active_now,
        lifecycle_edited,
        response,
    );
    super::sanitize_response_for_enabled(enabled, response);
}
