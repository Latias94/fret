use fret_core::MouseButton;
use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};

pub(in crate::imui) fn handle_pressable_drag_move_with_threshold(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    mv: fret_ui::action::PointerMoveCx,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<super::super::LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
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
            super::active_item::clear_active_item_for_target(host, acx, active_item_model);
            host.cancel_drag(mv.pointer_id);
            super::long_press_timer::cancel_for(host, long_press_signal_model);
            host.notify(acx);
            false
        }
        DragMoveOutcome::StartedDragging => {
            super::long_press_timer::cancel_for(host, long_press_signal_model);
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

pub(in crate::imui) fn finish_pressable_drag_on_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    up: fret_ui::action::PointerUpCx,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<super::super::LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
) {
    if up.button == MouseButton::Left {
        super::active_item::clear_active_item_for_target(host, acx, active_item_model);
        super::long_press_timer::cancel_for(host, long_press_signal_model);
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

pub(in crate::imui) fn prepare_pressable_drag_on_pointer_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    down: fret_ui::action::PointerDownCx,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<super::super::LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
) {
    if down.button != MouseButton::Left {
        return;
    }

    super::active_item::mark_active_item_for_target(host, acx, active_item_model);
    super::long_press_timer::arm_for(host, acx, long_press_signal_model);

    if host.drag(down.pointer_id).is_none() {
        host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
    }
}
