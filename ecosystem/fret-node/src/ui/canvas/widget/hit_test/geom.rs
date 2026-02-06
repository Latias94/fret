use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    // NOTE: Node bounds and port anchors must come from derived geometry (`CanvasGeometry`),
    // not ad-hoc layout guesses. See ADR 0135.

    pub(in super::super) fn rect_contains_point(rect: Rect, pos: Point) -> bool {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        pos.x.0 >= min_x && pos.x.0 <= max_x && pos.y.0 >= min_y && pos.y.0 <= max_y
    }

    pub(in super::super) fn distance_sq_point_to_rect(pos: Point, rect: Rect) -> f32 {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);

        let dx = if pos.x.0 < min_x {
            min_x - pos.x.0
        } else if pos.x.0 > max_x {
            pos.x.0 - max_x
        } else {
            0.0
        };
        let dy = if pos.y.0 < min_y {
            min_y - pos.y.0
        } else if pos.y.0 > max_y {
            pos.y.0 - max_y
        } else {
            0.0
        };

        dx * dx + dy * dy
    }
}
