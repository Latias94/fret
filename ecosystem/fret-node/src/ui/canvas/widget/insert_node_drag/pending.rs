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
    dnd_drop_canvas: DndItemId,
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
    let frame_id = cx.app.frame_id();
    let tick_id = cx.app.tick_id();

    ui_dnd::register_droppable_rect(
        cx.app.models_mut(),
        &dnd,
        window,
        frame_id,
        dnd_drop_canvas,
        cx.bounds,
        0,
        false,
    );
    let update = ui_dnd::handle_pointer_move_or_init_in_scope(
        cx.app.models_mut(),
        &dnd,
        window,
        frame_id,
        drag_kind,
        ui_dnd::DND_SCOPE_DEFAULT,
        pointer_id,
        pending.start_tick,
        start_window,
        current_window,
        tick_id,
        ActivationConstraint::Distance { px: 6.0 },
        CollisionStrategy::PointerWithin,
        Some((cx.bounds, AutoScrollConfig::default())),
    );
    if !matches!(update.sensor, SensorOutput::DragStart { .. }) {
        return false;
    }

    cx.app.begin_cross_window_drag_with_kind(
        pointer_id,
        drag_kind,
        window,
        start_window,
        super::InsertNodeDragPayload {
            candidate: pending.candidate.clone(),
        },
    );
    ui_dnd::clear_pointer(cx.app.models_mut(), &dnd, window, drag_kind, pointer_id);
    if let Some(drag) = cx.app.drag_mut(pointer_id)
        && drag.payload::<super::InsertNodeDragPayload>().is_some()
    {
        drag.dragging = true;
    }

    super::session::finish_pending_insert_node_drag(&mut canvas.interaction, cx);
    true
}
