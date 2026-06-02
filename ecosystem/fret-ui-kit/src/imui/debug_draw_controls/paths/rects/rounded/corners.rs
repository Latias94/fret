use fret_core::{Point, Px};

use crate::imui::debug_draw_controls::DebugDrawRoundCorners;

use super::super::super::sampling::append_arc_points;

pub(super) struct RectCornerRoundings {
    pub(super) top_left: Px,
    pub(super) top_right: Px,
    pub(super) bottom_right: Px,
    pub(super) bottom_left: Px,
}

impl RectCornerRoundings {
    pub(super) fn from_flags(rounding: Px, corners: DebugDrawRoundCorners) -> Self {
        Self {
            top_left: rounding_for_corner(rounding, corners, DebugDrawRoundCorners::TOP_LEFT),
            top_right: rounding_for_corner(rounding, corners, DebugDrawRoundCorners::TOP_RIGHT),
            bottom_right: rounding_for_corner(
                rounding,
                corners,
                DebugDrawRoundCorners::BOTTOM_RIGHT,
            ),
            bottom_left: rounding_for_corner(rounding, corners, DebugDrawRoundCorners::BOTTOM_LEFT),
        }
    }
}

pub(super) fn append_path_rect_corner_arc_points(
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

fn rounding_for_corner(
    rounding: Px,
    corners: DebugDrawRoundCorners,
    corner: DebugDrawRoundCorners,
) -> Px {
    if corners.contains(corner) {
        rounding
    } else {
        Px(0.0)
    }
}
