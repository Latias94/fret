use crate::core::{CanvasPoint, CanvasRect, CanvasSize};

pub(super) fn clamp_delta_for_extent_rect(
    delta: CanvasPoint,
    group_min: CanvasPoint,
    group_size: CanvasSize,
    extent: CanvasRect,
) -> CanvasPoint {
    let mut out = delta;

    let group_w = group_size.width.max(0.0);
    let group_h = group_size.height.max(0.0);
    let extent_w = extent.size.width.max(0.0);
    let extent_h = extent.size.height.max(0.0);

    if group_min.x.is_finite()
        && group_w.is_finite()
        && extent.origin.x.is_finite()
        && extent_w.is_finite()
    {
        let min_dx = extent.origin.x - group_min.x;
        let mut max_dx = extent.origin.x + (extent_w - group_w) - group_min.x;
        if min_dx.is_finite() {
            if !max_dx.is_finite() || max_dx < min_dx {
                max_dx = min_dx;
            }
            out.x = if out.x.is_finite() {
                out.x.clamp(min_dx, max_dx)
            } else {
                min_dx
            };
        }
    }

    if group_min.y.is_finite()
        && group_h.is_finite()
        && extent.origin.y.is_finite()
        && extent_h.is_finite()
    {
        let min_dy = extent.origin.y - group_min.y;
        let mut max_dy = extent.origin.y + (extent_h - group_h) - group_min.y;
        if min_dy.is_finite() {
            if !max_dy.is_finite() || max_dy < min_dy {
                max_dy = min_dy;
            }
            out.y = if out.y.is_finite() {
                out.y.clamp(min_dy, max_dy)
            } else {
                min_dy
            };
        }
    }

    out
}
