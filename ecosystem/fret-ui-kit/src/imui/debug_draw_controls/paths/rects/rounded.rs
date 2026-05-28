use fret_core::{Point, Px, Rect};

use super::super::sampling::append_arc_points;
use crate::imui::debug_draw_controls::DebugDrawRoundCorners;
use crate::imui::debug_draw_controls::geometry::effective_rect_rounding;

pub(in crate::imui::debug_draw_controls) fn append_path_rect_points(
    points: &mut Vec<Point>,
    rect: Rect,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) {
    let a = rect.origin;
    let b = rect_max_point(rect);
    let rounding = effective_rect_rounding(rect, rounding, corners);

    if rounding.0 < 0.5 {
        points.push(a);
        points.push(Point::new(b.x, a.y));
        points.push(b);
        points.push(Point::new(a.x, b.y));
        return;
    }

    let rounding_tl = if corners.contains(DebugDrawRoundCorners::TOP_LEFT) {
        rounding
    } else {
        Px(0.0)
    };
    let rounding_tr = if corners.contains(DebugDrawRoundCorners::TOP_RIGHT) {
        rounding
    } else {
        Px(0.0)
    };
    let rounding_br = if corners.contains(DebugDrawRoundCorners::BOTTOM_RIGHT) {
        rounding
    } else {
        Px(0.0)
    };
    let rounding_bl = if corners.contains(DebugDrawRoundCorners::BOTTOM_LEFT) {
        rounding
    } else {
        Px(0.0)
    };

    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(a.x.0 + rounding_tl.0), Px(a.y.0 + rounding_tl.0)),
        rounding_tl,
        6,
        9,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(b.x.0 - rounding_tr.0), Px(a.y.0 + rounding_tr.0)),
        rounding_tr,
        9,
        12,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(b.x.0 - rounding_br.0), Px(b.y.0 - rounding_br.0)),
        rounding_br,
        0,
        3,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(a.x.0 + rounding_bl.0), Px(b.y.0 - rounding_bl.0)),
        rounding_bl,
        3,
        6,
    );
}

fn append_path_rect_corner_arc_points(
    points: &mut Vec<Point>,
    center: Point,
    radius: Px,
    a_min_of_12: i32,
    a_max_of_12: i32,
) {
    if radius.0 < 0.5 {
        points.push(center);
        return;
    }
    let a_min = a_min_of_12 as f32 * std::f32::consts::TAU / 12.0;
    let a_max = a_max_of_12 as f32 * std::f32::consts::TAU / 12.0;
    append_arc_points(
        points,
        center,
        radius,
        a_min,
        a_max,
        a_min_of_12.abs_diff(a_max_of_12) as usize,
    );
}

fn rect_max_point(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0),
        Px(rect.origin.y.0 + rect.size.height.0),
    )
}
