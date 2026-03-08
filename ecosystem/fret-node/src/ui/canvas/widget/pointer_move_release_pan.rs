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
