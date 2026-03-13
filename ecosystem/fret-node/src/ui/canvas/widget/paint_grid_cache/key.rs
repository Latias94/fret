use crate::ui::canvas::widget::*;
use crate::ui::style::NodeGraphBackgroundPattern;

pub(super) fn build_grid_tile_cache_key(
    plan: &super::super::paint_grid_plan::GridPaintPlan,
) -> u64 {
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
