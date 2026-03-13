use super::coords::canvas_to_window;
use super::prelude::*;

pub(super) fn handle_pending_insert_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: MouseButtons,
    zoom: f32,
    drag_kind: DragKindId,
) -> bool {
    let Some(pending) = canvas.interaction.pending_insert_node_drag.clone() else {
        return false;
    };

    let Some(pointer_id) = cx.pointer_id else {
        return super::session::abort_pending_insert_node_drag(&mut canvas.interaction, cx);
    };
    if pending.pointer_id != pointer_id {
        return false;
    }

    if !buttons.left {
        return super::session::abort_pending_insert_node_drag_and_clear_dnd(
            &mut canvas.interaction,
            cx,
            drag_kind,
            pointer_id,
        );
    }

    if cx.window.is_none() {
        // Can't start an internal drag without a window id.
        return super::session::abort_pending_insert_node_drag(&mut canvas.interaction, cx);
    }

    let Some(window) = cx.window else {
        return false;
    };
    let start_window = canvas_to_window::<M>(cx.bounds, pending.start_pos, snapshot.pan, zoom);
    let current_window = canvas_to_window::<M>(cx.bounds, position, snapshot.pan, zoom);

    let dnd = ui_dnd::dnd_service_model_global(cx.app);
    let tick_id = cx.app.tick_id();
    let activation_probe = ui_dnd::DndActivationProbe::new(
        dnd.clone(),
        ui_dnd::DndActivationProbeConfig::for_kind(drag_kind)
            .activation_constraint(ActivationConstraint::Distance { px: 6.0 }),
    );
    let payload = super::InsertNodeDragPayload {
        candidate: pending.candidate.clone(),
    };
    let sensor = ui_dnd::try_begin_cross_window_drag_on_activation(
        cx.app,
        &activation_probe,
        window,
        pointer_id,
        pending.start_tick,
        start_window,
        current_window,
        tick_id,
        move |app| {
            fret_runtime::DragHost::begin_cross_window_drag_with_kind(
                app,
                pointer_id,
                drag_kind,
                window,
                start_window,
                payload,
            );
        },
    );
    if !matches!(sensor, SensorOutput::DragStart { .. }) {
        return false;
    }

    super::session::finish_pending_insert_node_drag(&mut canvas.interaction, cx);
    true
}
