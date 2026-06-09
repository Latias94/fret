use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ActiveTriggerBehavior, ActiveTriggerResponseInput};
use crate::imui::ResponseExt;

mod context_menu;
mod pressable;

pub(super) fn populate_active_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &ActiveTriggerBehavior,
    input: ActiveTriggerResponseInput,
    response: &mut ResponseExt,
) {
    context_menu::populate_active_trigger_context_menu_response(cx, id, behavior, response);
    pressable::populate_active_trigger_pressable_response(cx, id, state, behavior, input, response);
}
