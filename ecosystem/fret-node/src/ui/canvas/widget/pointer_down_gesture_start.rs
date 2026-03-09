use super::*;

pub(super) fn handle_close_button_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }

    let Some(command) = canvas.close_command.clone() else {
        return false;
    };
    let rect = NodeGraphCanvasWith::<M>::close_button_rect(snapshot.pan, zoom);
    if !NodeGraphCanvasWith::<M>::rect_contains(rect, position) {
        return false;
    }

    cx.dispatch_command(command);
    cx.stop_propagation();
    true
}

pub(super) fn handle_context_menu_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    canvas.interaction.context_menu.is_some()
        && context_menu::handle_context_menu_pointer_down(canvas, cx, position, button, zoom)
}

pub(super) fn handle_pending_right_click_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
) -> bool {
    if button != MouseButton::Right {
        return false;
    }

    cancel::cancel_active_gestures(canvas, cx);
    if !snapshot.interaction.pan_on_drag.right {
        return false;
    }

    canvas.interaction.pending_right_click = Some(crate::ui::canvas::state::PendingRightClick {
        start_pos: position,
    });
    cx.capture_pointer(cx.node);
    paint_invalidation::invalidate_paint(cx);
    true
}

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    sticky_wire::handle_sticky_wire_pointer_down(canvas, cx, snapshot, position, button, zoom)
}

pub(super) fn handle_pan_start_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
) -> bool {
    if button == MouseButton::Left
        && snapshot.interaction.space_to_pan
        && canvas.interaction.pan_activation_key_held
        && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
    {
        let _ =
            pan_zoom::begin_panning(canvas, cx, snapshot, position, fret_core::MouseButton::Left);
        return true;
    }

    if button == MouseButton::Middle && snapshot.interaction.pan_on_drag.middle {
        let _ = pan_zoom::begin_panning(
            canvas,
            cx,
            snapshot,
            position,
            fret_core::MouseButton::Middle,
        );
        return true;
    }

    false
}
