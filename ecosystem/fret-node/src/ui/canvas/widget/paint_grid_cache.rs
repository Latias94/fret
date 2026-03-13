#[path = "paint_grid_cache/key.rs"]
mod key;
#[path = "paint_grid_cache/ops.rs"]
mod ops;
#[path = "paint_grid_cache/warm.rs"]
mod warm;

use super::*;

pub(super) struct GridTileWarmupStats {
    pub(super) tile_budget_limit: u32,
    pub(super) tile_budget_used: u32,
    pub(super) skipped_tiles: u32,
}

pub(super) fn warm_grid_tiles<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    plan: &super::paint_grid_plan::GridPaintPlan,
    view_interacting: bool,
) -> GridTileWarmupStats {
    warm::warm_grid_tiles(canvas, cx, plan, view_interacting)
}
