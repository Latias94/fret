use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, item_behavior};

pub(super) fn populate_checkbox_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &item_behavior::PressableItemBehavior,
    enabled: bool,
    response: &mut ResponseExt,
) {
    let changed = cx.take_transient_for(id, crate::imui::KEY_CHANGED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        behavior,
        item_behavior::PressableItemResponseInput {
            enabled,
            clicked: false,
            changed,
            lifecycle_edited: changed,
        },
        response,
    );
}
