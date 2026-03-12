use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;
use crate::ui::canvas::state::NodeResizeHandle;

pub(super) fn close_button_rect(pan: CanvasPoint, zoom: f32) -> Rect {
    let margin = 12.0 / zoom;
    let width = 64.0 / zoom;
    let height = 24.0 / zoom;
    Rect::new(
        Point::new(Px(-pan.x + margin), Px(-pan.y + margin)),
        Size::new(Px(width), Px(height)),
    )
}

pub(super) fn rect_contains(rect: Rect, pos: Point) -> bool {
    pos.x.0 >= rect.origin.x.0
        && pos.y.0 >= rect.origin.y.0
        && pos.x.0 <= rect.origin.x.0 + rect.size.width.0
        && pos.y.0 <= rect.origin.y.0 + rect.size.height.0
}

pub(super) fn node_resize_handle_rect(
    node_rect: Rect,
    handle: NodeResizeHandle,
    zoom: f32,
    resize_handle_size: f32,
) -> Rect {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let min_size = 1.0 / zoom.max(1.0e-6);
    let size = (resize_handle_size / zoom).max(min_size);

    let max_w = (0.25 * node_rect.size.width.0.max(0.0)).max(min_size);
    let max_h = (0.25 * node_rect.size.height.0.max(0.0)).max(min_size);
    let size = size.min(max_w).min(max_h);

    let x0 = node_rect.origin.x.0;
    let y0 = node_rect.origin.y.0;
    let x1 = node_rect.origin.x.0 + node_rect.size.width.0;
    let y1 = node_rect.origin.y.0 + node_rect.size.height.0;

    let center_x = x0 + 0.5 * (x1 - x0 - size);
    let center_y = y0 + 0.5 * (y1 - y0 - size);

    let (x, y) = match handle {
        NodeResizeHandle::TopLeft => (x0, y0),
        NodeResizeHandle::Top => (center_x, y0),
        NodeResizeHandle::TopRight => (x1 - size, y0),
        NodeResizeHandle::Right => (x1 - size, center_y),
        NodeResizeHandle::BottomRight => (x1 - size, y1 - size),
        NodeResizeHandle::Bottom => (center_x, y1 - size),
        NodeResizeHandle::BottomLeft => (x0, y1 - size),
        NodeResizeHandle::Left => (x0, center_y),
    };

    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(size), Px(size)))
}

#[cfg(test)]
mod tests;
