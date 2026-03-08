use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

pub(super) struct GridTileSpec {
    pub(super) tile_origin: Point,
    pub(super) tile_size_canvas: f32,
    pub(super) spacing: f32,
    pub(super) major_every: i64,
    pub(super) major_color: Color,
    pub(super) minor_color: Color,
    pub(super) thickness: Px,
    pub(super) dot_size: f32,
    pub(super) cross_size: f32,
    pub(super) x0: i64,
    pub(super) x1: i64,
    pub(super) y0: i64,
    pub(super) y1: i64,
}

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
    let spec = GridTileSpec::new(
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
    let mut ops: Vec<SceneOp> = Vec::with_capacity(spec.approx_ops(pattern));

    match pattern {
        NodeGraphBackgroundPattern::Lines => {
            super::paint_grid_tiles_lines::append_line_ops(&spec, &mut ops)
        }
        NodeGraphBackgroundPattern::Dots => {
            super::paint_grid_tiles_dots::append_dot_ops(&spec, &mut ops)
        }
        NodeGraphBackgroundPattern::Cross => {
            super::paint_grid_tiles_cross::append_cross_ops(&spec, &mut ops)
        }
    }
    ops
}

impl GridTileSpec {
    pub(super) fn new(
        tile_origin: Point,
        tile_size_canvas: f32,
        spacing: f32,
        major_every: i64,
        major_color: Color,
        minor_color: Color,
        thickness: Px,
        dot_size: f32,
        cross_size: f32,
    ) -> Self {
        let tile_min_x = tile_origin.x.0;
        let tile_min_y = tile_origin.y.0;
        let tile_max_x = tile_min_x + tile_size_canvas;
        let tile_max_y = tile_min_y + tile_size_canvas;

        let x0 = (tile_min_x / spacing).floor() as i64;
        let x1 = (tile_max_x / spacing).ceil() as i64;
        let y0 = (tile_min_y / spacing).floor() as i64;
        let y1 = (tile_max_y / spacing).ceil() as i64;

        Self {
            tile_origin,
            tile_size_canvas,
            spacing,
            major_every,
            major_color,
            minor_color,
            thickness,
            dot_size,
            cross_size,
            x0,
            x1,
            y0,
            y1,
        }
    }

    pub(super) fn approx_ops(&self, pattern: NodeGraphBackgroundPattern) -> usize {
        let approx_x = (self.x1 - self.x0 + 1).max(0) as usize;
        let approx_y = (self.y1 - self.y0 + 1).max(0) as usize;
        let approx_points = approx_x.saturating_mul(approx_y);
        match pattern {
            NodeGraphBackgroundPattern::Lines => approx_x.saturating_add(approx_y),
            NodeGraphBackgroundPattern::Dots => approx_points,
            NodeGraphBackgroundPattern::Cross => approx_points.saturating_mul(2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::grid_tile_ops;
    use crate::ui::style::NodeGraphBackgroundPattern;
    use fret_core::{Color, DrawOrder, Edges, Px};

    #[test]
    fn dots_pattern_emits_rounded_quads() {
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let ops = grid_tile_ops(
            NodeGraphBackgroundPattern::Dots,
            fret_core::Point::new(Px(0.0), Px(0.0)),
            100.0,
            20.0,
            4,
            white,
            white,
            Px(1.0),
            2.0,
            6.0,
        );

        assert!(!ops.is_empty());
        let any_rounded = ops.iter().any(|op| match op {
            fret_core::SceneOp::Quad { corner_radii, .. } => {
                corner_radii.top_left.0 > 0.0
                    || corner_radii.top_right.0 > 0.0
                    || corner_radii.bottom_left.0 > 0.0
                    || corner_radii.bottom_right.0 > 0.0
            }
            _ => false,
        });
        assert!(any_rounded);
    }

    #[test]
    fn cross_pattern_emits_axis_aligned_segments() {
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let ops = grid_tile_ops(
            NodeGraphBackgroundPattern::Cross,
            fret_core::Point::new(Px(0.0), Px(0.0)),
            40.0,
            20.0,
            4,
            white,
            white,
            Px(1.0),
            1.0,
            6.0,
        );

        assert!(!ops.is_empty());
        assert!(ops.iter().all(|op| matches!(
            op,
            fret_core::SceneOp::Quad {
                order: DrawOrder(1),
                border: Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: Px(0.0),
                    left: Px(0.0)
                },
                ..
            }
        )));
    }
}
