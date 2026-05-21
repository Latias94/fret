use fret_runtime::Platform;
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, searcher::SearcherCx, viewport_motion_cx::ViewportMotionCx,
};

pub(super) trait PointerWheelPlatformCx {
    fn platform(&self) -> Platform;
}

pub(super) trait PointerWheelViewportCx<H: UiHost>:
    ViewportMotionCx<H> + PointerWheelPlatformCx
{
}

impl<H, T> PointerWheelViewportCx<H> for T
where
    H: UiHost,
    T: ViewportMotionCx<H> + PointerWheelPlatformCx,
{
}

pub(super) trait PointerWheelCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerWheelViewportCx<H> + SearcherCx<H, M>
{
}

impl<H, M, T> PointerWheelCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerWheelViewportCx<H> + SearcherCx<H, M>,
{
}
