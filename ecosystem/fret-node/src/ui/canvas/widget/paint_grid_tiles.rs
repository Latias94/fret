use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

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
    let tile_min_x = tile_origin.x.0;
    let tile_min_y = tile_origin.y.0;
    let tile_max_x = tile_min_x + tile_size_canvas;
    let tile_max_y = tile_min_y + tile_size_canvas;

    let x0 = (tile_min_x / spacing).floor() as i64;
    let x1 = (tile_max_x / spacing).ceil() as i64;
    let y0 = (tile_min_y / spacing).floor() as i64;
    let y1 = (tile_max_y / spacing).ceil() as i64;

    let approx_x = (x1 - x0 + 1).max(0) as usize;
    let approx_y = (y1 - y0 + 1).max(0) as usize;
    let approx_points = approx_x.saturating_mul(approx_y);
    let approx_ops = match pattern {
        NodeGraphBackgroundPattern::Lines => approx_x.saturating_add(approx_y),
        NodeGraphBackgroundPattern::Dots => approx_points,
        NodeGraphBackgroundPattern::Cross => approx_points.saturating_mul(2),
    };
    let mut ops: Vec<SceneOp> = Vec::with_capacity(approx_ops);

    match pattern {
        NodeGraphBackgroundPattern::Lines => {
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let color = if ix.rem_euclid(major_every) == 0 {
                    major_color
                } else {
                    minor_color
                };
                ops.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(x - tile_origin.x.0 - 0.5 * thickness.0), Px(0.0)),
                        Size::new(thickness, Px(tile_size_canvas)),
                    ),
                    background: fret_core::Paint::Solid(color).into(),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),

                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            for iy in y0..=y1 {
                let y = iy as f32 * spacing;
                let color = if iy.rem_euclid(major_every) == 0 {
                    major_color
                } else {
                    minor_color
                };
                ops.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(0.0), Px(y - tile_origin.y.0 - 0.5 * thickness.0)),
                        Size::new(Px(tile_size_canvas), thickness),
                    ),
                    background: fret_core::Paint::Solid(color).into(),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),

                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }
        NodeGraphBackgroundPattern::Dots => {
            let d = dot_size.max(0.0);
            let r = 0.5 * d;
            if !(d.is_finite() && d > 0.0) {
                return ops;
            }

            let corner = Corners::all(Px(r));
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let x_local = x - tile_origin.x.0;
                for iy in y0..=y1 {
                    let y = iy as f32 * spacing;
                    let y_local = y - tile_origin.y.0;

                    let is_major =
                        ix.rem_euclid(major_every) == 0 && iy.rem_euclid(major_every) == 0;
                    let color = if is_major { major_color } else { minor_color };

                    ops.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: Rect::new(
                            Point::new(Px(x_local - r), Px(y_local - r)),
                            Size::new(Px(d), Px(d)),
                        ),
                        background: fret_core::Paint::Solid(color).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: corner,
                    });
                }
            }
        }
        NodeGraphBackgroundPattern::Cross => {
            let s = cross_size.max(0.0);
            if !(s.is_finite() && s > 0.0) {
                return ops;
            }

            let half = 0.5 * s;
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let x_local = x - tile_origin.x.0;
                for iy in y0..=y1 {
                    let y = iy as f32 * spacing;
                    let y_local = y - tile_origin.y.0;

                    let is_major =
                        ix.rem_euclid(major_every) == 0 || iy.rem_euclid(major_every) == 0;
                    let color = if is_major { major_color } else { minor_color };

                    // Vertical segment.
                    ops.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: Rect::new(
                            Point::new(Px(x_local - 0.5 * thickness.0), Px(y_local - half)),
                            Size::new(thickness, Px(s)),
                        ),
                        background: fret_core::Paint::Solid(color).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(0.0)),
                    });
                    // Horizontal segment.
                    ops.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: Rect::new(
                            Point::new(Px(x_local - half), Px(y_local - 0.5 * thickness.0)),
                            Size::new(Px(s), thickness),
                        ),
                        background: fret_core::Paint::Solid(color).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }
    }

    ops
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
