use fret_core::{Point, Px, Rect};

use super::super::DebugDrawRoundCorners;

pub(in crate::imui::debug_draw_controls) fn rect_is_empty(rect: Rect) -> bool {
    rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0
}

pub(in crate::imui::debug_draw_controls) fn rect_is_finite(rect: Rect) -> bool {
    rect.origin.x.0.is_finite()
        && rect.origin.y.0.is_finite()
        && rect.size.width.0.is_finite()
        && rect.size.height.0.is_finite()
}

pub(in crate::imui::debug_draw_controls) fn rect_quad_points(rect: Rect) -> [Point; 4] {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = Px(rect.origin.x.0 + rect.size.width.0);
    let y1 = Px(rect.origin.y.0 + rect.size.height.0);
    [
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ]
}

pub(in crate::imui::debug_draw_controls) fn effective_rect_rounding(
    rect: Rect,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) -> Px {
    if rect_is_empty(rect)
        || !rect_is_finite(rect)
        || !rounding.0.is_finite()
        || rounding.0 < 0.5
        || corners.is_empty()
    {
        return Px(0.0);
    }

    let width = rect.size.width.0.abs();
    let height = rect.size.height.0.abs();
    let x_scale = if corners.contains(DebugDrawRoundCorners::TOP)
        || corners.contains(DebugDrawRoundCorners::BOTTOM)
    {
        0.5
    } else {
        1.0
    };
    let y_scale = if corners.contains(DebugDrawRoundCorners::LEFT)
        || corners.contains(DebugDrawRoundCorners::RIGHT)
    {
        0.5
    } else {
        1.0
    };
    let rounding = rounding
        .0
        .min(width * x_scale - 1.0)
        .min(height * y_scale - 1.0);
    if rounding >= 0.5 {
        Px(rounding)
    } else {
        Px(0.0)
    }
}
