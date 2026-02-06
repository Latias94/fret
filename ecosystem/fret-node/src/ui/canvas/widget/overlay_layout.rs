use super::*;

fn clamp_overlay_origin_in_visible_canvas_rect(desired: Point, size: Size, visible: Rect) -> Point {
    let min_x = visible.origin.x.0;
    let min_y = visible.origin.y.0;
    let max_x = visible.origin.x.0 + (visible.size.width.0 - size.width.0).max(0.0);
    let max_y = visible.origin.y.0 + (visible.size.height.0 - size.height.0).max(0.0);

    Point::new(
        Px(desired.x.0.clamp(min_x, max_x)),
        Px(desired.y.0.clamp(min_y, max_y)),
    )
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn clamp_context_menu_origin(
        &self,
        desired: Point,
        item_count: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let size = context_menu_size_at_zoom(&self.style, item_count, snapshot.zoom);

        let viewport = Self::viewport_from_snapshot(bounds, snapshot);
        let vis = viewport.visible_canvas_rect();
        clamp_overlay_origin_in_visible_canvas_rect(desired, size, vis)
    }

    pub(super) fn clamp_searcher_origin(
        &self,
        desired: Point,
        visible_rows: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let size = searcher_size_at_zoom(&self.style, visible_rows, snapshot.zoom);

        let viewport = Self::viewport_from_snapshot(bounds, snapshot);
        let vis = viewport.visible_canvas_rect();
        clamp_overlay_origin_in_visible_canvas_rect(desired, size, vis)
    }
}
