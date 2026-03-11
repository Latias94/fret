use super::super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

pub(in super::super) struct GridTileSpec {
    pub(in super::super) tile_origin: Point,
    pub(in super::super) tile_size_canvas: f32,
    pub(in super::super) spacing: f32,
    pub(in super::super) major_every: i64,
    pub(in super::super) major_color: Color,
    pub(in super::super) minor_color: Color,
    pub(in super::super) thickness: Px,
    pub(in super::super) dot_size: f32,
    pub(in super::super) cross_size: f32,
    pub(in super::super) x0: i64,
    pub(in super::super) x1: i64,
    pub(in super::super) y0: i64,
    pub(in super::super) y1: i64,
}

pub(in super::super) fn grid_tile_spec(
    tile_origin: Point,
    tile_size_canvas: f32,
    spacing: f32,
    major_every: i64,
    major_color: Color,
    minor_color: Color,
    thickness: Px,
    dot_size: f32,
    cross_size: f32,
) -> GridTileSpec {
    let tile_min_x = tile_origin.x.0;
    let tile_min_y = tile_origin.y.0;
    let tile_max_x = tile_min_x + tile_size_canvas;
    let tile_max_y = tile_min_y + tile_size_canvas;

    let x0 = (tile_min_x / spacing).floor() as i64;
    let x1 = (tile_max_x / spacing).ceil() as i64;
    let y0 = (tile_min_y / spacing).floor() as i64;
    let y1 = (tile_max_y / spacing).ceil() as i64;

    GridTileSpec {
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

pub(in super::super) fn approx_ops(
    spec: &GridTileSpec,
    pattern: NodeGraphBackgroundPattern,
) -> usize {
    let approx_x = (spec.x1 - spec.x0 + 1).max(0) as usize;
    let approx_y = (spec.y1 - spec.y0 + 1).max(0) as usize;
    let approx_points = approx_x.saturating_mul(approx_y);
    match pattern {
        NodeGraphBackgroundPattern::Lines => approx_x.saturating_add(approx_y),
        NodeGraphBackgroundPattern::Dots => approx_points,
        NodeGraphBackgroundPattern::Cross => approx_points.saturating_mul(2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    #[test]
    fn grid_tile_spec_projects_tile_indices_from_bounds() {
        let spec = grid_tile_spec(
            Point::new(Px(5.0), Px(15.0)),
            40.0,
            20.0,
            4,
            white(),
            white(),
            Px(1.0),
            2.0,
            6.0,
        );

        assert_eq!(spec.x0, 0);
        assert_eq!(spec.x1, 3);
        assert_eq!(spec.y0, 0);
        assert_eq!(spec.y1, 3);
    }

    #[test]
    fn approx_ops_matches_pattern_density() {
        let spec = grid_tile_spec(
            Point::new(Px(0.0), Px(0.0)),
            40.0,
            20.0,
            4,
            white(),
            white(),
            Px(1.0),
            2.0,
            6.0,
        );

        assert_eq!(approx_ops(&spec, NodeGraphBackgroundPattern::Lines), 6);
        assert_eq!(approx_ops(&spec, NodeGraphBackgroundPattern::Dots), 9);
        assert_eq!(approx_ops(&spec, NodeGraphBackgroundPattern::Cross), 18);
    }
}
