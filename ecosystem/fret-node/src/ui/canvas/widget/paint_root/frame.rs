#[path = "frame/background.rs"]
mod background;
#[path = "frame/cache.rs"]
mod cache;

use crate::ui::canvas::widget::*;

pub(super) struct PaintRootFrameViewport {
    pub(super) viewport_rect: Rect,
    pub(super) viewport_w: f32,
    pub(super) viewport_h: f32,
    pub(super) viewport_origin_x: f32,
    pub(super) viewport_origin_y: f32,
    pub(super) render_cull_rect: Option<Rect>,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn prepare_paint_root_frame<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        view_interacting: bool,
    ) -> PaintRootFrameViewport {
        cache::begin_paint_root_caches(self);
        cache::record_path_cache_stats(self, cx);

        let viewport = Self::viewport_from_pan_zoom(cx.bounds, snapshot.pan, snapshot.zoom);
        let viewport_rect = viewport.visible_canvas_rect();
        let viewport_w = viewport_rect.size.width.0;
        let viewport_h = viewport_rect.size.height.0;
        let viewport_origin_x = viewport_rect.origin.x.0;
        let viewport_origin_y = viewport_rect.origin.y.0;
        let render_cull_rect = self.compute_render_cull_rect(snapshot, cx.bounds);

        cx.scene.push(SceneOp::PushClipRect {
            rect: viewport_rect,
        });

        background::paint_canvas_background(self, cx, viewport_rect);

        self.paint_grid(
            cx,
            viewport_rect,
            render_cull_rect,
            snapshot.zoom,
            view_interacting,
        );

        PaintRootFrameViewport {
            viewport_rect,
            viewport_w,
            viewport_h,
            viewport_origin_x,
            viewport_origin_y,
            render_cull_rect,
        }
    }
}
