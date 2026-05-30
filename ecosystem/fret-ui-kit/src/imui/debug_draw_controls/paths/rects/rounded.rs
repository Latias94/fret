use fret_core::{Point, Px, Rect};

use crate::imui::debug_draw_controls::DebugDrawRoundCorners;
use crate::imui::debug_draw_controls::geometry::effective_rect_rounding;

mod corners;
mod geometry;

use corners::{RectCornerRoundings, append_path_rect_corner_arc_points};
use geometry::rect_max_point;

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

    let roundings = RectCornerRoundings::from_flags(rounding, corners);

    append_path_rect_corner_arc_points(
        points,
        Point::new(
            Px(a.x.0 + roundings.top_left.0),
            Px(a.y.0 + roundings.top_left.0),
        ),
        roundings.top_left,
        6,
        9,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(
            Px(b.x.0 - roundings.top_right.0),
            Px(a.y.0 + roundings.top_right.0),
        ),
        roundings.top_right,
        9,
        12,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(
            Px(b.x.0 - roundings.bottom_right.0),
            Px(b.y.0 - roundings.bottom_right.0),
        ),
        roundings.bottom_right,
        0,
        3,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(
            Px(a.x.0 + roundings.bottom_left.0),
            Px(b.y.0 - roundings.bottom_left.0),
        ),
        roundings.bottom_left,
        3,
        6,
    );
}
