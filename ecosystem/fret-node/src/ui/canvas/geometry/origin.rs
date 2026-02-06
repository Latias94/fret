use crate::core::{CanvasPoint, CanvasSize};
use crate::io::NodeGraphNodeOrigin;

pub(crate) fn node_origin_offset_canvas(
    size_canvas: CanvasSize,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    let origin = node_origin.normalized();
    CanvasPoint {
        x: origin.x * size_canvas.width,
        y: origin.y * size_canvas.height,
    }
}

pub(crate) fn node_rect_origin_from_anchor(
    anchor: CanvasPoint,
    size_canvas: CanvasSize,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    let off = node_origin_offset_canvas(size_canvas, node_origin);
    CanvasPoint {
        x: anchor.x - off.x,
        y: anchor.y - off.y,
    }
}

pub(crate) fn node_anchor_from_rect_origin(
    rect_origin: CanvasPoint,
    size_canvas: CanvasSize,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    let off = node_origin_offset_canvas(size_canvas, node_origin);
    CanvasPoint {
        x: rect_origin.x + off.x,
        y: rect_origin.y + off.y,
    }
}
