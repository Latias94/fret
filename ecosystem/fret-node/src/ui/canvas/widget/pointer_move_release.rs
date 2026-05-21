use fret_core::Point;
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    pan_zoom_begin_cx::PanZoomBeginCx, pointer_up_cx::PointerUpCx,
};

pub(super) trait PointerMoveReleaseCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerUpCx<H, M> + PanZoomBeginCx<H>
{
}

impl<H, M, T> PointerMoveReleaseCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerUpCx<H, M> + PanZoomBeginCx<H>,
{
}

pub(super) fn handle_missing_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerMoveReleaseCx<H, M>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_move_release_pan::handle_missing_pan_release(
        canvas, cx, position, buttons, modifiers,
    )
}

pub(super) fn handle_pending_right_click_pan_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerMoveReleaseCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    zoom: f32,
) -> bool {
    super::pointer_move_release_pan::handle_pending_right_click_pan_start(
        canvas, cx, snapshot, position, buttons, zoom,
    )
}

pub(super) fn handle_missing_left_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerUpCx<H, M>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_move_release_left::handle_missing_left_release(
        canvas, cx, position, buttons, modifiers,
    )
}
