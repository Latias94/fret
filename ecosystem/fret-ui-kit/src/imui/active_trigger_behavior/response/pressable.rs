use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    ResponseExt, active_trigger_behavior::ActiveTriggerBehavior,
    active_trigger_behavior::ActiveTriggerResponseInput, install_hover_query_hooks_for_pressable,
    populate_pressable_response,
};

pub(super) fn populate_active_trigger_pressable_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &ActiveTriggerBehavior,
    input: ActiveTriggerResponseInput,
    response: &mut ResponseExt,
) {
    let hover_delay = install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
    populate_pressable_response(
        cx,
        id,
        state,
        hover_delay,
        &behavior.active_item_model,
        input.clicked,
        input.changed,
        state.pressed,
        input.lifecycle_edited,
        input.enabled,
        response,
    );
}
