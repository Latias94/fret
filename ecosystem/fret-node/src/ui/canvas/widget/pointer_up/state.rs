use super::*;

pub(super) fn sync_pointer_up_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: fret_core::Modifiers,
) {
    super::super::pointer_up_state::sync_pointer_up_state(canvas, position, modifiers);
}
