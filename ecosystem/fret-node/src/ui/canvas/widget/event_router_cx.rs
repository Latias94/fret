use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, event_pointer_down_route_cx::PointerDownRouteCx,
    event_pointer_up::PointerUpRouteCx, event_router_pointer_wheel_cx::PointerWheelRouteCx,
    event_router_system::SystemRouteCx, pointer_move_release::PointerMoveReleaseCx,
    pointer_move_tail_cx::PointerMoveTailCx,
};

pub(super) trait PointerButtonRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerDownRouteCx<H, M>
    + PointerMoveReleaseCx<H, M>
    + PointerMoveTailCx<H>
    + PointerUpRouteCx<H, M>
{
}

impl<H, M, T> PointerButtonRouteCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerDownRouteCx<H, M>
        + PointerMoveReleaseCx<H, M>
        + PointerMoveTailCx<H>
        + PointerUpRouteCx<H, M>,
{
}

pub(super) trait PointerEventRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerButtonRouteCx<H, M> + PointerWheelRouteCx<H, M>
{
}

impl<H, M, T> PointerEventRouteCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerButtonRouteCx<H, M> + PointerWheelRouteCx<H, M>,
{
}

pub(super) trait EventRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    SystemRouteCx<H, M> + PointerEventRouteCx<H, M>
{
}

impl<H, M, T> EventRouteCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: SystemRouteCx<H, M> + PointerEventRouteCx<H, M>,
{
}
