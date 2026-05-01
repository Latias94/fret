use fret_core::{CursorIcon, MouseButton, Point, PointerId, Px};
use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Default)]
struct DragReportState {
    last_position: Option<Point>,
}

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
            let _ = host.update_model(active_item_model, |st| {
                if st.active == Some(acx.target) {
                    st.active = None;
                }
            });
            host.cancel_drag(mv.pointer_id);
            cancel_long_press_timer_for(host, long_press_signal_model);
            host.notify(acx);
            false
        }
        DragMoveOutcome::StartedDragging => {
            cancel_long_press_timer_for(host, long_press_signal_model);
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
        let _ = host.update_model(active_item_model, |st| {
            if st.active == Some(acx.target) {
                st.active = None;
            }
        });
        cancel_long_press_timer_for(host, long_press_signal_model);
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

pub(in super::super) fn populate_pressable_drag_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    response: &mut super::super::ResponseExt,
) {
    response.drag.started = cx.take_transient_for(id, super::super::KEY_DRAG_STARTED);
    response.drag.stopped = cx.take_transient_for(id, super::super::KEY_DRAG_STOPPED);
    response.drag.dragging = false;
    response.drag.delta = Point::default();
    response.drag.total = Point::default();

    let kind = drag_kind_for_element(id);
    let pointer_id = cx.app.find_drag_pointer_id(|d| {
        d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
    });
    let drag_snapshot = pointer_id.and_then(|pointer_id| {
        cx.app
            .drag(pointer_id)
            .filter(|drag| drag.kind == kind)
            .map(|drag| (drag.dragging, drag.position, drag.start_position))
    });
    if let Some((dragging, current, start)) = drag_snapshot {
        response.drag.dragging = dragging;
        let (delta, total) = cx.state_for(id, DragReportState::default, |st| {
            let prev = st.last_position.unwrap_or(current);
            st.last_position = Some(current);
            (
                super::super::point_sub(current, prev),
                super::super::point_sub(current, start),
            )
        });
        if dragging {
            response.drag.delta = delta;
            response.drag.total = total;
        }
    } else {
        cx.state_for(id, DragReportState::default, |st| {
            st.last_position = None;
        });
    }
}

pub(in super::super) fn mark_active_item_on_left_pointer_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    active_item_model: &fret_runtime::Model<super::ImUiActiveItemState>,
    request_focus: bool,
) {
    if button != MouseButton::Left {
        return;
    }
    if request_focus {
        host.request_focus(acx.target);
    }
    let _ = host.update_model(active_item_model, |st| {
        st.active = Some(acx.target);
    });
    host.notify(acx);
}

pub(in super::super) fn clear_active_item_on_left_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    active_item_model: &fret_runtime::Model<super::ImUiActiveItemState>,
) {
    if button != MouseButton::Left {
        return;
    }
    let _ = host.update_model(active_item_model, |st| {
        if st.active == Some(acx.target) {
            st.active = None;
        }
    });
    host.notify(acx);
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

    let _ = host.update_model(active_item_model, |st| {
        st.active = Some(acx.target);
    });
    arm_long_press_timer_for(host, acx, long_press_signal_model);

    if host.drag(down.pointer_id).is_none() {
        host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
    }
}

pub(in super::super) fn prepare_pointer_region_drag_on_left_down(
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

pub(in super::super) fn handle_pointer_region_drag_move_with_threshold(
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
                host.record_transient_event(acx, super::super::KEY_DRAG_STOPPED);
            }
            host.cancel_drag(mv.pointer_id);
            host.release_pointer_capture();
            host.notify(acx);
            return false;
        }
        DragMoveOutcome::StartedDragging => {
            host.record_transient_event(acx, super::super::KEY_DRAG_STARTED);
        }
        DragMoveOutcome::Continue => {}
    }

    host.notify(acx);
    false
}

pub(in super::super) fn finish_pointer_region_drag(
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
            host.record_transient_event(acx, super::super::KEY_DRAG_STOPPED);
        }
        host.cancel_drag(pointer_id);
    }
    host.release_pointer_capture();
    host.notify(acx);
    false
}

fn arm_long_press_timer_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<super::LongPressSignalState>,
) {
    let token = host.next_timer_token();
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.timer = Some(token);
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: super::super::LONG_PRESS_DELAY,
        repeat: None,
    });
}

fn cancel_long_press_timer_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    model: &fret_runtime::Model<super::LongPressSignalState>,
) {
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
}
