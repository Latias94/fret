//! Private shared behavior for active-only immediate-mode triggers.

use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

pub(super) struct ActiveTriggerBehavior {
    pub(super) active_item_model: Model<ImUiActiveItemState>,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ActiveTriggerBehaviorOptions {
    pub(super) request_focus_on_press: bool,
    pub(super) clear_pointer_move: bool,
}

impl Default for ActiveTriggerBehaviorOptions {
    fn default() -> Self {
        Self {
            request_focus_on_press: true,
            clear_pointer_move: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ActiveTriggerResponseInput {
    pub(super) enabled: bool,
    pub(super) clicked: bool,
    pub(super) changed: bool,
    pub(super) lifecycle_edited: bool,
}

pub(super) fn install_active_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    options: ActiveTriggerBehaviorOptions,
) -> ActiveTriggerBehavior {
    cx.pressable_clear_on_pointer_down();
    if options.clear_pointer_move {
        cx.pressable_clear_on_pointer_move();
    }
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::active_item_model_for_window(cx);
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_up = active_item_model.clone();
    let lifecycle_model = super::lifecycle_session_model_for(cx, id);
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model.clone();
    let request_focus_on_press = options.request_focus_on_press;

    cx.pressable_on_pointer_down(std::sync::Arc::new(move |host, acx, down| {
        super::mark_lifecycle_activated_on_left_pointer_down(
            host,
            acx,
            down.button,
            &lifecycle_model_for_down,
        );
        super::mark_active_item_on_left_pointer_down(
            host,
            acx,
            down.button,
            &active_item_model_for_down,
            request_focus_on_press,
        );
        PressablePointerDownResult::Continue
    }));

    cx.pressable_on_pointer_up(std::sync::Arc::new(move |host, acx, up| {
        super::mark_lifecycle_deactivated_on_left_pointer_up(
            host,
            acx,
            up.button,
            &lifecycle_model_for_up,
        );
        super::clear_active_item_on_left_pointer_up(
            host,
            acx,
            up.button,
            &active_item_model_for_up,
        );
        PressablePointerUpResult::Continue
    }));

    ActiveTriggerBehavior {
        active_item_model,
        lifecycle_model,
    }
}

pub(super) fn populate_active_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &ActiveTriggerBehavior,
    input: ActiveTriggerResponseInput,
    response: &mut super::ResponseExt,
) {
    response.core.hovered = state.hovered;
    response.core.pressed = state.pressed;
    response.core.focused = state.focused;
    response.nav_highlighted =
        state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
    response.id = Some(id);
    response.core.clicked = input.clicked;
    response.core.changed = input.changed;
    response.core.rect = cx.last_bounds_for_element(id);
    let hover_delay =
        super::install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
    response.pointer_hovered_raw = state.hovered_raw;
    response.pointer_hovered_raw_below_barrier = state.hovered_raw_below_barrier;
    response.hover_stationary_met = hover_delay.stationary_met;
    response.hover_delay_short_met = hover_delay.delay_short_met;
    response.hover_delay_normal_met = hover_delay.delay_normal_met;
    response.hover_delay_short_shared_met = hover_delay.shared_delay_short_met;
    response.hover_delay_normal_shared_met = hover_delay.shared_delay_normal_met;
    response.hover_blocked_by_active_item =
        super::hover_blocked_by_active_item_for(cx, id, &behavior.active_item_model);
    super::populate_response_lifecycle_transients(cx, id, response);
    super::populate_response_lifecycle_from_active_state(
        cx,
        id,
        state.pressed,
        input.lifecycle_edited,
        response,
    );
    super::sanitize_response_for_enabled(input.enabled, response);
}
