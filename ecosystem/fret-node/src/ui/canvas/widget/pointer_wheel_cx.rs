use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, searcher::SearcherCx, viewport_motion_cx::ViewportMotionCx,
};

pub(super) trait PointerWheelCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    ViewportMotionCx<H> + SearcherCx<H, M>
{
}

impl<H, M, T> PointerWheelCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: ViewportMotionCx<H> + SearcherCx<H, M>,
{
}
