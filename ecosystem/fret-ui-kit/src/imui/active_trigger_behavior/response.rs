use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ActiveTriggerBehavior, ActiveTriggerResponseInput};
use crate::imui::{
    KEY_CONTEXT_MENU_REQUESTED, KEY_SECONDARY_CLICKED, ResponseExt,
    install_hover_query_hooks_for_pressable, populate_pressable_response,
};

pub(super) fn populate_active_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &ActiveTriggerBehavior,
    input: ActiveTriggerResponseInput,
    response: &mut ResponseExt,
) {
    response.set_secondary_clicked(cx.take_transient_for(id, KEY_SECONDARY_CLICKED));
    response.set_context_menu_requested(cx.take_transient_for(id, KEY_CONTEXT_MENU_REQUESTED));
    response.set_context_menu_anchor(
        cx.read_model(
            &behavior.context_anchor_model,
            fret_ui::Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(None),
    );
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
