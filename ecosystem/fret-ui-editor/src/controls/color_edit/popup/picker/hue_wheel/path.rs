use fret_core::{PathCommand, Point, Px};

use super::super::super::super::model::HueWheelTriangle;

pub(super) const HUE_WHEEL_TRIANGLE_STEPS: usize = 12;

pub(super) fn triangle_grid_barycentric(i: usize, j: usize) -> (f32, f32, f32) {
    let n = HUE_WHEEL_TRIANGLE_STEPS as f32;
    let u = i as f32 / n;
    let v = j as f32 / n;
    (u, v, (1.0 - u - v).max(0.0))
}

pub(super) fn point_from_triangle_barycentric(
    triangle: HueWheelTriangle,
    barycentric: (f32, f32, f32),
) -> (f32, f32) {
    (
        triangle.hue.0 * barycentric.0
            + triangle.black.0 * barycentric.1
            + triangle.white.0 * barycentric.2,
        triangle.hue.1 * barycentric.0
            + triangle.black.1 * barycentric.1
            + triangle.white.1 * barycentric.2,
    )
}

pub(super) fn absolute_point(origin: Point, local: (f32, f32)) -> Point {
    Point::new(Px(origin.x.0 + local.0), Px(origin.y.0 + local.1))
}

pub(super) fn triangle_path(a: Point, b: Point, c: Point) -> [PathCommand; 4] {
    [
        PathCommand::MoveTo(a),
        PathCommand::LineTo(b),
        PathCommand::LineTo(c),
        PathCommand::Close,
    ]
}

pub(super) fn circle_path(center: Point, radius: f32) -> [PathCommand; 6] {
    let r = radius.max(0.0);
    let k = 0.552_284_8_f32 * r;
    let cx = center.x.0;
    let cy = center.y.0;
    [
        PathCommand::MoveTo(Point::new(Px(cx + r), Px(cy))),
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + r), Px(cy + k)),
            ctrl2: Point::new(Px(cx + k), Px(cy + r)),
            to: Point::new(Px(cx), Px(cy + r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - k), Px(cy + r)),
            ctrl2: Point::new(Px(cx - r), Px(cy + k)),
            to: Point::new(Px(cx - r), Px(cy)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - r), Px(cy - k)),
            ctrl2: Point::new(Px(cx - k), Px(cy - r)),
            to: Point::new(Px(cx), Px(cy - r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + k), Px(cy - r)),
            ctrl2: Point::new(Px(cx + r), Px(cy - k)),
            to: Point::new(Px(cx + r), Px(cy)),
        },
        PathCommand::Close,
    ]
}
