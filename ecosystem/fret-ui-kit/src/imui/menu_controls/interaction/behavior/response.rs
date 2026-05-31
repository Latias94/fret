use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{KEY_CLICKED, ResponseExt, active_trigger_behavior};

pub(in crate::imui::menu_controls::interaction) fn populate_menu_item_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    enabled: bool,
    response: &mut ResponseExt,
) {
    let clicked = cx.take_transient_for(id, KEY_CLICKED);
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
