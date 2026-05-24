use super::*;

pub(super) fn prepare_pointer_down_state<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::super::event_pointer_down_route_cx::PointerDownRouteCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
) {
    super::super::event_pointer_down_state::prepare_pointer_down_state(
        canvas, cx, snapshot, position, modifiers,
    );
}
