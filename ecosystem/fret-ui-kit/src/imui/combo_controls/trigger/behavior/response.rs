use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, item_behavior};

pub(super) fn populate_combo_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &item_behavior::PressableItemBehavior,
    enabled: bool,
    response: &mut ResponseExt,
) {
    let clicked = cx.take_transient_for(id, crate::imui::KEY_CLICKED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        behavior,
        item_behavior::PressableItemResponseInput {
            enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
