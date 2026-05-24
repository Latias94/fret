use crate::ui::canvas::widget::*;

pub(super) fn handle_close_button_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl pointer_down_close_button_cx::PointerDownCloseButtonCx<H>,
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

    cx.dispatch_close_command(command);
    cx.stop_propagation();
    true
}
