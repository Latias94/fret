use crate::ui::canvas::widget::*;

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
