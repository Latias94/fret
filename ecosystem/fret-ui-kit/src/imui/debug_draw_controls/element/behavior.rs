use fret_ui::action::ActivateReason;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{KEY_CLICKED, ResponseExt, item_behavior, mark_lifecycle_instant_if_inactive};

pub(super) fn install_debug_draw_pressable_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    enabled: bool,
    response: &mut ResponseExt,
) {
    let behavior = item_behavior::install_pressable_item_behavior_with_options(
        cx,
        id,
        item_behavior::PressableItemBehaviorOptions {
            report_pointer_click: true,
        },
    );
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
        }
        host.record_transient_event(acx, KEY_CLICKED);
        host.notify(acx);
    }));

    let clicked = cx.take_transient_for(id, KEY_CLICKED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        &behavior,
        item_behavior::PressableItemResponseInput {
            enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
