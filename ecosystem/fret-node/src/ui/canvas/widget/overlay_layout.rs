use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn clamp_context_menu_origin(
        &self,
        desired: Point,
        item_count: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = context_menu_rect_at(&self.style, desired, item_count, snapshot.zoom);

        let viewport = CanvasViewport2D::new(
            bounds,
            PanZoom2D {
                pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
                zoom: snapshot.zoom,
            },
        );
        let vis = viewport.visible_canvas_rect();

        let min_x = vis.origin.x.0;
        let min_y = vis.origin.y.0;
        let max_x = vis.origin.x.0 + (vis.size.width.0 - rect.size.width.0).max(0.0);
        let max_y = vis.origin.y.0 + (vis.size.height.0 - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }

    pub(super) fn clamp_searcher_origin(
        &self,
        desired: Point,
        visible_rows: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = searcher_rect_at(&self.style, desired, visible_rows, snapshot.zoom);

        let viewport = CanvasViewport2D::new(
            bounds,
            PanZoom2D {
                pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
                zoom: snapshot.zoom,
            },
        );
        let vis = viewport.visible_canvas_rect();

        let min_x = vis.origin.x.0;
        let min_y = vis.origin.y.0;
        let max_x = vis.origin.x.0 + (vis.size.width.0 - rect.size.width.0).max(0.0);
        let max_y = vis.origin.y.0 + (vis.size.height.0 - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }
}
