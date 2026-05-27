use fret_core::{CursorIcon, MouseButton, PointerId};
use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};

pub(in crate::imui) fn prepare_pointer_region_drag_on_left_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    down: fret_ui::action::PointerDownCx,
    drag_kind: Option<fret_runtime::DragKindId>,
    cursor: Option<CursorIcon>,
) -> bool {
    if down.button != MouseButton::Left {
        return false;
    }

    host.request_focus(acx.target);
    if let Some(cursor) = cursor {
        host.set_cursor_icon(cursor);
    }
    if let Some(drag_kind) = drag_kind {
        host.capture_pointer();
        if host.drag(down.pointer_id).is_none() {
            host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
        }
    }
    true
}

pub(in crate::imui) fn handle_pointer_region_drag_move_with_threshold(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    mv: fret_ui::action::PointerMoveCx,
    drag_kind: fret_runtime::DragKindId,
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

pub(in crate::imui) fn finish_pointer_region_drag(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    pointer_id: PointerId,
    drag_kind: fret_runtime::DragKindId,
) -> bool {
    if let Some(drag) = host.drag(pointer_id)
        && drag.kind == drag_kind
        && drag.source_window == acx.window
    {
        if drag.dragging {
            host.record_transient_event(acx, crate::imui::KEY_DRAG_STOPPED);
        }
        host.cancel_drag(pointer_id);
    }
    host.release_pointer_capture();
    host.notify(acx);
    false
}
