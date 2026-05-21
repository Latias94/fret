mod missing_release;
mod pending_right_click;

use super::*;

pub(super) fn handle_missing_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool
where
    Cx: pointer_move_release::PointerMoveReleaseCx<H, M>,
{
    missing_release::handle_missing_pan_release(canvas, cx, position, buttons, modifiers)
}

pub(super) fn handle_pending_right_click_pan_start<H: UiHost, M: NodeGraphCanvasMiddleware, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    zoom: f32,
) -> bool
where
    Cx: pointer_move_release::PointerMoveReleaseCx<H, M>,
{
    pending_right_click::handle_pending_right_click_pan_start(
        canvas, cx, snapshot, position, buttons, zoom,
    )
}
