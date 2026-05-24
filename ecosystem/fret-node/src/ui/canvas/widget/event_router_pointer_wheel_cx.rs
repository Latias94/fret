use fret_runtime::Platform;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, pointer_wheel_cx::PointerWheelCx};

pub(super) trait PointerWheelRoutePlatformCx {
    fn platform(&self) -> Platform;
}

pub(super) trait PointerWheelRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerWheelCx<H, M> + PointerWheelRoutePlatformCx
{
}

impl<H, M, T> PointerWheelRouteCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerWheelCx<H, M> + PointerWheelRoutePlatformCx,
{
}
