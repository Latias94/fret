//! Paint-root frame viewport adapter contract.
//!
//! This module keeps bounds-driven viewport and render-cull calculation behind a named seam.
//! Diagnostics and scene emission remain in frame setup.

use fret_core::Rect;
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

use super::frame::PaintRootFrameViewport;

pub(super) trait PaintRootFrameViewportCx<H: UiHost> {
    fn paint_root_frame_bounds(&self) -> Rect;
}

pub(super) fn prepare_paint_root_frame_viewport<H, M>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &impl PaintRootFrameViewportCx<H>,
    snapshot: &ViewSnapshot,
) -> PaintRootFrameViewport
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    let bounds = cx.paint_root_frame_bounds();
    let viewport =
        NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(bounds, snapshot.pan, snapshot.zoom);
    let viewport_rect = viewport.visible_canvas_rect();
    let viewport_w = viewport_rect.size.width.0;
    let viewport_h = viewport_rect.size.height.0;
    let viewport_origin_x = viewport_rect.origin.x.0;
    let viewport_origin_y = viewport_rect.origin.y.0;
    let render_cull_rect = canvas.compute_render_cull_rect(snapshot, bounds);

    PaintRootFrameViewport {
        viewport_rect,
        viewport_w,
        viewport_h,
        viewport_origin_x,
        viewport_origin_y,
        render_cull_rect,
    }
}
