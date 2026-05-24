use super::*;

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::PointerDownStartCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    sticky_wire::handle_sticky_wire_pointer_down(canvas, cx, snapshot, position, button, zoom)
}
