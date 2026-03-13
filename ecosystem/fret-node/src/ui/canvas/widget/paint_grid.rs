use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_grid<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        viewport_rect: Rect,
        render_cull_rect: Option<Rect>,
        zoom: f32,
        view_interacting: bool,
    ) {
        self.grid_scene_cache.begin_frame();

        let Some(plan) = super::paint_grid_plan::prepare_grid_paint(
            self,
            cx,
            viewport_rect,
            render_cull_rect,
            zoom,
        ) else {
            return;
        };

        let warmup = super::paint_grid_cache::warm_grid_tiles(self, cx, &plan, view_interacting);
        super::paint_grid_stats::record_grid_tile_cache_stats(self, cx, &warmup);
    }
}
