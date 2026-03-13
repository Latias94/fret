use crate::ui::canvas::widget::*;

pub(super) fn cache_tile_rect(tile: TileCoord, tile_size_canvas: f32) -> Rect {
    let tile_origin = tile.origin(tile_size_canvas);
    Rect::new(
        tile_origin,
        Size::new(Px(tile_size_canvas), Px(tile_size_canvas)),
    )
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn collect_sorted_edge_cache_tiles(
        &mut self,
        edges_rect: Rect,
        viewport_rect: Rect,
        tile_size_canvas: f32,
    ) -> Vec<TileCoord> {
        let tiles = TileGrid2D::new(tile_size_canvas);
        tiles.tiles_in_rect(edges_rect, &mut self.edges_tiles_scratch);
        tiles.sort_tiles_center_first(viewport_rect, &mut self.edges_tiles_scratch);
        self.edges_tiles_scratch.clone()
    }

    pub(super) fn cache_tile_cull_rect(&self, tile_rect: Rect, zoom: f32) -> Rect {
        let margin_screen = self.style.paint.render_cull_margin_px;
        if margin_screen.is_finite() && margin_screen > 0.0 {
            inflate_rect(tile_rect, margin_screen / zoom)
        } else {
            tile_rect
        }
    }
}
