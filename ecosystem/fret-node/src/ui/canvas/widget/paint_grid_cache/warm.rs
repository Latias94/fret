use super::*;

pub(super) fn warm_grid_tiles<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    plan: &super::super::paint_grid_plan::GridPaintPlan,
    view_interacting: bool,
) -> GridTileWarmupStats {
    let tile_budget_limit =
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME
            .select(view_interacting);
    let mut tile_budget = WorkBudget::new(tile_budget_limit);
    let tile_size_canvas = plan.tile_size_canvas;
    let warmup = warm_scene_op_tiles_u64_with(
        &mut canvas.grid_scene_cache,
        cx.scene,
        &canvas.grid_tiles_scratch,
        super::key::build_grid_tile_cache_key(plan),
        1,
        &mut tile_budget,
        |tile| tile.origin(tile_size_canvas),
        |_ops| {},
        |tile| super::ops::grid_tile_ops_for_plan(plan, tile),
    );
    if warmup.skipped_tiles > 0 {
        super::super::redraw_request::request_paint_redraw(cx);
    }
    GridTileWarmupStats {
        tile_budget_limit,
        tile_budget_used: tile_budget.used(),
        skipped_tiles: warmup.skipped_tiles,
    }
}
