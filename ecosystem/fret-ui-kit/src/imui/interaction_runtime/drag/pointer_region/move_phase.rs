use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};
use fret_runtime::DragKindId;
use fret_ui::action::{ActionCx, PointerMoveCx, UiPointerActionHost};

pub(in crate::imui) fn handle_pointer_region_drag_move_with_threshold(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    mv: PointerMoveCx,
    drag_kind: DragKindId,
    drag_threshold: InteractionDragThreshold,
) -> bool {
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
    match outcome {
        DragMoveOutcome::Canceled => {
            if was_dragging {
                host.record_transient_event(acx, crate::imui::KEY_DRAG_STOPPED);
            }
            host.cancel_drag(mv.pointer_id);
            host.release_pointer_capture();
            host.notify(acx);
            return false;
        }
        DragMoveOutcome::StartedDragging => {
            host.record_transient_event(acx, crate::imui::KEY_DRAG_STARTED);
        }
        DragMoveOutcome::Continue => {}
    }

    host.notify(acx);
    false
}
