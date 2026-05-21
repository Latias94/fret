use super::*;

pub(super) fn dispatch_pointer_move_tail<H: UiHost, M: NodeGraphCanvasMiddleware, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) where
    Cx: super::super::pointer_move_tail_cx::PointerMoveTailCx<H>,
{
    super::super::event_pointer_move_tail::dispatch_pointer_move_tail(
        canvas, cx, snapshot, position, buttons, modifiers, zoom,
    );
}
