use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, active_trigger_behavior};

pub(super) fn populate_tab_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    enabled: bool,
    response: &mut ResponseExt,
) {
    let clicked = cx.take_transient_for(id, crate::imui::KEY_CLICKED);
    active_trigger_behavior::populate_active_trigger_response(
        cx,
        id,
        state,
        behavior,
        active_trigger_behavior::ActiveTriggerResponseInput {
            enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
