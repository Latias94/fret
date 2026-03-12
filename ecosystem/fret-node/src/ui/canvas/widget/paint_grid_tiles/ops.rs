use super::*;

pub(super) fn grid_tile_ops(
    pattern: NodeGraphBackgroundPattern,
    tile_origin: Point,
    tile_size_canvas: f32,
    spacing: f32,
    major_every: i64,
    major_color: Color,
    minor_color: Color,
    thickness: Px,
    dot_size: f32,
    cross_size: f32,
) -> Vec<SceneOp> {
    let spec = super::support::grid_tile_spec(
        tile_origin,
        tile_size_canvas,
        spacing,
        major_every,
        major_color,
        minor_color,
        thickness,
        dot_size,
        cross_size,
    );
    let mut ops: Vec<SceneOp> = Vec::with_capacity(super::support::approx_ops(&spec, pattern));

    match pattern {
        NodeGraphBackgroundPattern::Lines => {
            super::super::paint_grid_tiles_lines::append_line_ops(&spec, &mut ops)
        }
        NodeGraphBackgroundPattern::Dots => {
            super::super::paint_grid_tiles_dots::append_dot_ops(&spec, &mut ops)
        }
        NodeGraphBackgroundPattern::Cross => {
            super::super::paint_grid_tiles_cross::append_cross_ops(&spec, &mut ops)
        }
    }
    ops
}
