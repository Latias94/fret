use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{ActionCx, PointerUpCx, UiPointerActionHost};

use super::super::super::{ImUiActiveItemState, LongPressSignalState};
use super::super::{active_item, long_press_timer};

pub(in crate::imui) fn finish_pressable_drag_on_pointer_up(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    up: PointerUpCx,
    active_item_model: &Model<ImUiActiveItemState>,
    long_press_signal_model: &Model<LongPressSignalState>,
    drag_kind: DragKindId,
) {
    if up.button == MouseButton::Left {
        active_item::clear_active_item_for_target(host, acx, active_item_model);
        long_press_timer::cancel_for(host, long_press_signal_model);
    }

    if let Some(drag) = host.drag(up.pointer_id)
        && drag.kind == drag_kind
        && drag.source_window == acx.window
    {
        if drag.dragging {
            host.record_transient_event(acx, crate::imui::KEY_DRAG_STOPPED);
        }
        host.cancel_drag(up.pointer_id);
        host.notify(acx);
    }
}
