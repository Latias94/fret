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
    response.core.hovered = state.hovered;
    response.core.pressed = state.pressed;
    response.core.focused = state.focused;
    response.nav_highlighted =
        state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
    response.id = Some(id);
    response.core.clicked = clicked;
    response.core.changed = changed;
    response.core.rect = cx.last_bounds_for_element(id);
    response.pointer_hovered_raw = state.hovered_raw;
    response.pointer_hovered_raw_below_barrier = state.hovered_raw_below_barrier;
    response.hover_stationary_met = hover_delay.stationary_met;
    response.hover_delay_short_met = hover_delay.delay_short_met;
    response.hover_delay_normal_met = hover_delay.delay_normal_met;
    response.hover_delay_short_shared_met = hover_delay.shared_delay_short_met;
    response.hover_delay_normal_shared_met = hover_delay.shared_delay_normal_met;
    response.hover_blocked_by_active_item =
        super::hover_blocked_by_active_item_for(cx, id, active_item_model);
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
