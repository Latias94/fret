use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{ActionCx, PointerDownCx, UiPointerActionHost};

use super::super::super::{ImUiActiveItemState, LongPressSignalState};
use super::super::{active_item, long_press_timer};

pub(in crate::imui) fn prepare_pressable_drag_on_pointer_down(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    down: PointerDownCx,
    active_item_model: &Model<ImUiActiveItemState>,
    long_press_signal_model: &Model<LongPressSignalState>,
    drag_kind: DragKindId,
) {
    if down.button != MouseButton::Left {
        return;
    }

    active_item::mark_active_item_for_target(host, acx, active_item_model);
    long_press_timer::arm_for(host, acx, long_press_signal_model);

    if host.drag(down.pointer_id).is_none() {
        host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
    }
}
