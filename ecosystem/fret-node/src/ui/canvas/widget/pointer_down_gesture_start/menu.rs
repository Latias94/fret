use super::*;

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
