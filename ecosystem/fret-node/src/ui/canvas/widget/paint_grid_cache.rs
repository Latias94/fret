use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

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
    let tile_budget_limit =
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME
            .select(view_interacting);
    let mut tile_budget = WorkBudget::new(tile_budget_limit);
    let tile_size_canvas = plan.tile_size_canvas;
    let warmup = warm_scene_op_tiles_u64_with(
        &mut canvas.grid_scene_cache,
        cx.scene,
        &canvas.grid_tiles_scratch,
        build_grid_tile_cache_key(plan),
        1,
        &mut tile_budget,
        |tile| tile.origin(tile_size_canvas),
        |_ops| {},
        |tile| grid_tile_ops_for_plan(plan, tile),
    );
    if warmup.skipped_tiles > 0 {
        cx.request_redraw();
    }
    GridTileWarmupStats {
        tile_budget_limit,
        tile_budget_used: tile_budget.used(),
        skipped_tiles: warmup.skipped_tiles,
    }
}

fn grid_tile_ops_for_plan(
    plan: &super::paint_grid_plan::GridPaintPlan,
    tile: TileCoord,
) -> Vec<SceneOp> {
    let tile_origin = tile.origin(plan.tile_size_canvas);
    super::paint_grid_tiles::grid_tile_ops(
        plan.pattern,
        tile_origin,
        plan.tile_size_canvas,
        plan.spacing,
        plan.major_every,
        plan.major_color,
        plan.minor_color,
        plan.thickness,
        plan.dot_size,
        plan.cross_size,
    )
}

fn build_grid_tile_cache_key(plan: &super::paint_grid_plan::GridPaintPlan) -> u64 {
    let mut builder = TileCacheKeyBuilder::new("fret-node.grid.tile.v1");
    builder.add_f32_bits(plan.zoom);
    builder.add_f32_bits(plan.tile_size_canvas);
    builder.add_u32(pattern_tag(plan.pattern));
    builder.add_u32(plan.spacing.to_bits());
    builder.add_u32(plan.line_width_px.to_bits());
    builder.add_u32(plan.thickness.0.to_bits());
    builder.add_u32(plan.dot_size.to_bits());
    builder.add_u32(plan.cross_size.to_bits());
    builder.add_i64(plan.major_every);
    builder.add_u32(plan.major_color.r.to_bits());
    builder.add_u32(plan.major_color.g.to_bits());
    builder.add_u32(plan.major_color.b.to_bits());
    builder.add_u32(plan.major_color.a.to_bits());
    builder.add_u32(plan.minor_color.r.to_bits());
    builder.add_u32(plan.minor_color.g.to_bits());
    builder.add_u32(plan.minor_color.b.to_bits());
    builder.add_u32(plan.minor_color.a.to_bits());
    builder.finish()
}

fn pattern_tag(pattern: NodeGraphBackgroundPattern) -> u32 {
    match pattern {
        NodeGraphBackgroundPattern::Lines => 0,
        NodeGraphBackgroundPattern::Dots => 1,
        NodeGraphBackgroundPattern::Cross => 2,
    }
}
