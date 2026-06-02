use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{ActionCx, PointerMoveCx, UiPointerActionHost};

use super::super::super::{ImUiActiveItemState, LongPressSignalState};
use super::super::{active_item, long_press_timer};

pub(in crate::imui) fn handle_pressable_drag_move_with_threshold(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    mv: PointerMoveCx,
    active_item_model: &Model<ImUiActiveItemState>,
    long_press_signal_model: &Model<LongPressSignalState>,
    drag_kind: DragKindId,
    drag_threshold: InteractionDragThreshold,
) -> bool {
    let (outcome, was_dragging) = {
        let Some(drag) = host.drag_mut(mv.pointer_id) else {
            return false;
        };
        if drag.kind != drag_kind || drag.source_window != acx.window {
            return false;
        }

        let was_dragging = drag.dragging;
        let outcome = update_thresholded_move(
            drag,
            acx.window,
            mv.position,
            mv.buttons.left,
            drag_threshold,
        );
        (outcome, was_dragging)
    };

    match outcome {
        DragMoveOutcome::Canceled => {
            if was_dragging {
                host.record_transient_event(acx, crate::imui::KEY_DRAG_STOPPED);
            }
            active_item::clear_active_item_for_target(host, acx, active_item_model);
            host.cancel_drag(mv.pointer_id);
            long_press_timer::cancel_for(host, long_press_signal_model);
            host.notify(acx);
            false
        }
        DragMoveOutcome::StartedDragging => {
            long_press_timer::cancel_for(host, long_press_signal_model);
            host.record_transient_event(acx, crate::imui::KEY_DRAG_STARTED);
            host.notify(acx);
            false
        }
        DragMoveOutcome::Continue => {
            host.notify(acx);
            false
        }
    }
}
