mod ops;
mod support;
#[cfg(test)]
mod tests;

use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

pub(super) use support::GridTileSpec;

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
    ops::grid_tile_ops(
        pattern,
        tile_origin,
        tile_size_canvas,
        spacing,
        major_every,
        major_color,
        minor_color,
        thickness,
        dot_size,
        cross_size,
    )
}
