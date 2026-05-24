use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, event_pointer_down_route::PointerDownPreflightCx,
    event_pointer_down_state_cx::PointerDownStateCx, left_click::LeftClickCx,
    pointer_down_gesture_start::PointerDownStartCx, right_click::RightClickCx,
};

pub(super) trait PointerDownRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerDownStateCx<H>
    + PointerDownPreflightCx<H>
    + PointerDownStartCx<H, M>
    + RightClickCx<H, M>
    + LeftClickCx<H>
{
}

impl<H, M, T> PointerDownRouteCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerDownStateCx<H>
        + PointerDownPreflightCx<H>
        + PointerDownStartCx<H, M>
        + RightClickCx<H, M>
        + LeftClickCx<H>,
{
}
