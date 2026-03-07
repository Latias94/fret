use super::*;

pub(super) fn handle_missing_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !canvas.interaction.panning {
        return false;
    }

    let should_end = match canvas.interaction.panning_button {
        Some(fret_core::MouseButton::Middle) => !buttons.middle,
        Some(fret_core::MouseButton::Left) => !buttons.left,
        Some(fret_core::MouseButton::Right) => !buttons.right,
        _ => false,
    };
    if !should_end {
        return false;
    }

    let snapshot = canvas.sync_view_state(cx.app);
    let button = canvas
        .interaction
        .panning_button
        .unwrap_or(fret_core::MouseButton::Middle);
    let _ = pointer_up::handle_pointer_up(
        canvas,
        cx,
        &snapshot,
        position,
        button,
        1,
        modifiers,
        snapshot.zoom,
    );
    true
}

pub(super) fn handle_pending_right_click_pan_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    zoom: f32,
) -> bool {
    if !(snapshot.interaction.pan_on_drag.right
        && buttons.right
        && canvas.interaction.panning_button.is_none())
    {
        return false;
    }

    let Some(pending) = canvas.interaction.pending_right_click else {
        return false;
    };

    if !right_click::pending_right_click_exceeded_drag_threshold(
        pending,
        position,
        snapshot.interaction.pane_click_distance,
        zoom,
    ) {
        return false;
    }

    canvas.interaction.pending_right_click = None;
    let _ = pan_zoom::begin_panning(
        canvas,
        cx,
        snapshot,
        position,
        fret_core::MouseButton::Right,
    );
    true
}

pub(super) fn handle_missing_left_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    let has_left_interaction = canvas.interaction.pending_marquee.is_some()
        || canvas.interaction.marquee.is_some()
        || canvas.interaction.pending_node_drag.is_some()
        || canvas.interaction.node_drag.is_some()
        || canvas.interaction.pending_group_drag.is_some()
        || canvas.interaction.group_drag.is_some()
        || canvas.interaction.pending_group_resize.is_some()
        || canvas.interaction.group_resize.is_some()
        || canvas.interaction.pending_node_resize.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.pending_wire_drag.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.pending_edge_insert_drag.is_some()
        || canvas.interaction.edge_insert_drag.is_some()
        || canvas.interaction.edge_drag.is_some();

    if !has_left_interaction || buttons.left {
        return false;
    }

    let snapshot = canvas.sync_view_state(cx.app);
    let _ = pointer_up::handle_pointer_up(
        canvas,
        cx,
        &snapshot,
        position,
        fret_core::MouseButton::Left,
        1,
        modifiers,
        snapshot.zoom,
    );
    true
}

pub(super) fn seed_or_update_last_pointer_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    if canvas.interaction.last_pos.is_none() {
        canvas.interaction.last_pos = Some(position);
        canvas.interaction.last_modifiers = modifiers;
        canvas.interaction.last_canvas_pos = Some(CanvasPoint {
            x: position.x.0,
            y: position.y.0,
        });
        return true;
    }

    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.last_canvas_pos = Some(CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    });
    false
}
