use fret_core::{MouseButton, Px};
use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod active_item;
mod long_press_timer;
mod pointer_region;
mod response;

pub(in super::super) use active_item::{
    clear_active_item_on_left_pointer_up, mark_active_item_on_left_pointer_down,
};
pub(in super::super) use pointer_region::{
    finish_pointer_region_drag, handle_pointer_region_drag_move_with_threshold,
    prepare_pointer_region_drag_on_left_down,
};
pub(in super::super) use response::populate_pressable_drag_response;

pub(in super::super) fn drag_kind_for_element(
    element: GlobalElementId,
) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(super::super::DRAG_KIND_MASK | element.0)
}

pub(in super::super) fn drag_threshold_for<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> InteractionDragThreshold {
    let theme = fret_ui::Theme::global(&*cx.app);
    let px = theme
        .metric_by_key(crate::theme_tokens::metric::COMPONENT_IMUI_DRAG_THRESHOLD_PX)
        .unwrap_or(Px(super::super::DEFAULT_DRAG_THRESHOLD_PX));
    InteractionDragThreshold::new(px)
}

pub(in super::super) fn handle_pressable_drag_move_with_threshold(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    mv: fret_ui::action::PointerMoveCx,
    active_item_model: &fret_runtime::Model<super::ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<super::LongPressSignalState>,
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
                host.record_transient_event(acx, super::super::KEY_DRAG_STOPPED);
            }
            active_item::clear_active_item_for_target(host, acx, active_item_model);
            host.cancel_drag(mv.pointer_id);
            long_press_timer::cancel_for(host, long_press_signal_model);
            host.notify(acx);
            false
        }
        DragMoveOutcome::StartedDragging => {
            long_press_timer::cancel_for(host, long_press_signal_model);
            host.record_transient_event(acx, super::super::KEY_DRAG_STARTED);
            host.notify(acx);
            false
        }
        DragMoveOutcome::Continue => {
            host.notify(acx);
            false
        }
    }
}

pub(in super::super) fn finish_pressable_drag_on_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    up: fret_ui::action::PointerUpCx,
    active_item_model: &fret_runtime::Model<super::ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<super::LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
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
            host.record_transient_event(acx, super::super::KEY_DRAG_STOPPED);
        }
        host.cancel_drag(up.pointer_id);
        host.notify(acx);
    }
}

pub(in super::super) fn prepare_pressable_drag_on_pointer_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    down: fret_ui::action::PointerDownCx,
    active_item_model: &fret_runtime::Model<super::ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<super::LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
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
