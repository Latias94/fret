use fret_core::{Corners, Px, Rect};

use crate::imui::debug_draw_controls::DebugDrawRoundCorners;
use crate::imui::debug_draw_controls::geometry::effective_rect_rounding;

pub(in crate::imui::debug_draw_controls) fn corner_radii_are_visible(radii: Corners) -> bool {
    radii.top_left.0 > 0.0
        || radii.top_right.0 > 0.0
        || radii.bottom_right.0 > 0.0
        || radii.bottom_left.0 > 0.0
}

pub(in crate::imui::debug_draw_controls) fn rounded_rect_corner_radii(
    rect: Rect,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) -> Corners {
    let rounding = effective_rect_rounding(rect, rounding, corners);
    Corners {
        top_left: if corners.contains(DebugDrawRoundCorners::TOP_LEFT) {
            rounding
        } else {
            Px(0.0)
        },
        top_right: if corners.contains(DebugDrawRoundCorners::TOP_RIGHT) {
            rounding
        } else {
            Px(0.0)
        },
        bottom_right: if corners.contains(DebugDrawRoundCorners::BOTTOM_RIGHT) {
            rounding
        } else {
            Px(0.0)
        },
        bottom_left: if corners.contains(DebugDrawRoundCorners::BOTTOM_LEFT) {
            rounding
        } else {
            Px(0.0)
        },
    }
}
