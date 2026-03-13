use crate::ui::canvas::widget::*;

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
