use crate::core::{CanvasPoint, CanvasRect, CanvasSize};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::geometry::{node_anchor_from_rect_origin, node_rect_origin_from_anchor};

fn clamp_rect_origin_in_rect_with_size(
    rect_origin: CanvasPoint,
    size: CanvasSize,
    extent: CanvasRect,
) -> CanvasPoint {
    let mut out = rect_origin;
    let node_w = size.width.max(0.0);
    let node_h = size.height.max(0.0);

    let min_x = extent.origin.x;
    let min_y = extent.origin.y;
    let max_x = extent.origin.x + (extent.size.width - node_w).max(0.0);
    let max_y = extent.origin.y + (extent.size.height - node_h).max(0.0);
    out.x = out.x.clamp(min_x, max_x);
    out.y = out.y.clamp(min_y, max_y);
    out
}

pub(super) fn clamp_anchor_in_rect_with_size(
    anchor: CanvasPoint,
    size: CanvasSize,
    extent: CanvasRect,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    let rect_origin = node_rect_origin_from_anchor(anchor, size, node_origin);
    let clamped = clamp_rect_origin_in_rect_with_size(rect_origin, size, extent);
    node_anchor_from_rect_origin(clamped, size, node_origin)
}

pub(super) fn union_rect(a: CanvasRect, b: CanvasRect) -> CanvasRect {
    let ax0 = a.origin.x;
    let ay0 = a.origin.y;
    let ax1 = a.origin.x + a.size.width;
    let ay1 = a.origin.y + a.size.height;

    let bx0 = b.origin.x;
    let by0 = b.origin.y;
    let bx1 = b.origin.x + b.size.width;
    let by1 = b.origin.y + b.size.height;

    let min_x = ax0.min(bx0);
    let min_y = ay0.min(by0);
    let max_x = ax1.max(bx1);
    let max_y = ay1.max(by1);

    CanvasRect {
        origin: CanvasPoint { x: min_x, y: min_y },
        size: CanvasSize {
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        },
    }
}
