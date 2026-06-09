use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    KEY_CONTEXT_MENU_REQUESTED, KEY_SECONDARY_CLICKED, ResponseExt,
    active_trigger_behavior::ActiveTriggerBehavior,
};

pub(super) fn populate_active_trigger_context_menu_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    behavior: &ActiveTriggerBehavior,
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
}
