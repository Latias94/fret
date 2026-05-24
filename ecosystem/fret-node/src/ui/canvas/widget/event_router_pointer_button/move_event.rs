use super::*;

pub(super) fn route_pointer_move_event<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: super::super::pointer_move_release::PointerMoveReleaseCx<H, M>
        + super::super::pointer_move_tail_cx::PointerMoveTailCx<H>,
{
    let Event::Pointer(fret_core::PointerEvent::Move {
        position,
        buttons,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    canvas.handle_pointer_move(cx, snapshot, *position, *buttons, *modifiers, zoom);
    true
}
