use super::*;

pub(super) fn populate_grid_tiles<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    viewport_rect: Rect,
    grid_rect: Rect,
    tile_size_canvas: f32,
) {
    let grid_tiles = TileGrid2D::new(tile_size_canvas);
    grid_tiles.tiles_in_rect(grid_rect, &mut canvas.grid_tiles_scratch);
    grid_tiles.sort_tiles_center_first(viewport_rect, &mut canvas.grid_tiles_scratch);
}
