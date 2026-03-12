use super::*;

pub(super) fn grid_tile_ops_for_plan(
    plan: &super::super::paint_grid_plan::GridPaintPlan,
    tile: TileCoord,
) -> Vec<SceneOp> {
    let tile_origin = tile.origin(plan.tile_size_canvas);
    super::super::paint_grid_tiles::grid_tile_ops(
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
