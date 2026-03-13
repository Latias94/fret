use crate::ui::canvas::widget::*;

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
