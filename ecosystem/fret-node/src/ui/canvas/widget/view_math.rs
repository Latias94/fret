use super::*;
use fret_canvas::view::{PanZoomConstraints2D, clamp_pan_zoom_view};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn viewport_from_pan_zoom(
        bounds: Rect,
        pan: CanvasPoint,
        zoom: f32,
    ) -> CanvasViewport2D {
        CanvasViewport2D::new(
            bounds,
            PanZoom2D {
                pan: Point::new(Px(pan.x), Px(pan.y)),
                zoom,
            },
        )
    }

    pub(super) fn viewport_from_snapshot(
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> CanvasViewport2D {
        Self::viewport_from_pan_zoom(bounds, snapshot.pan, snapshot.zoom)
    }

    pub(super) fn close_button_rect(pan: CanvasPoint, zoom: f32) -> Rect {
        let margin = 12.0 / zoom;
        let w = 64.0 / zoom;
        let h = 24.0 / zoom;
        Rect::new(
            Point::new(Px(-pan.x + margin), Px(-pan.y + margin)),
            Size::new(Px(w), Px(h)),
        )
    }

    pub(super) fn rect_contains(rect: Rect, pos: Point) -> bool {
        pos.x.0 >= rect.origin.x.0
            && pos.y.0 >= rect.origin.y.0
            && pos.x.0 <= rect.origin.x.0 + rect.size.width.0
            && pos.y.0 <= rect.origin.y.0 + rect.size.height.0
    }

    pub(super) fn resize_handle_rect(&self, node_rect: Rect, zoom: f32) -> Rect {
        self.node_resize_handle_rect(node_rect, NodeResizeHandle::BottomRight, zoom)
    }

    pub(crate) fn node_resize_handle_rect(
        &self,
        node_rect: Rect,
        handle: NodeResizeHandle,
        zoom: f32,
    ) -> Rect {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let min_size = 1.0 / zoom.max(1.0e-6);
        let size = (self.style.resize_handle_size / zoom).max(min_size);

        let max_w = (0.25 * node_rect.size.width.0.max(0.0)).max(min_size);
        let max_h = (0.25 * node_rect.size.height.0.max(0.0)).max(min_size);
        let size = size.min(max_w).min(max_h);

        let x0 = node_rect.origin.x.0;
        let y0 = node_rect.origin.y.0;
        let x1 = node_rect.origin.x.0 + node_rect.size.width.0;
        let y1 = node_rect.origin.y.0 + node_rect.size.height.0;

        let cx = x0 + 0.5 * (x1 - x0 - size);
        let cy = y0 + 0.5 * (y1 - y0 - size);

        let (x, y) = match handle {
            NodeResizeHandle::TopLeft => (x0, y0),
            NodeResizeHandle::Top => (cx, y0),
            NodeResizeHandle::TopRight => (x1 - size, y0),
            NodeResizeHandle::Right => (x1 - size, cy),
            NodeResizeHandle::BottomRight => (x1 - size, y1 - size),
            NodeResizeHandle::Bottom => (cx, y1 - size),
            NodeResizeHandle::BottomLeft => (x0, y1 - size),
            NodeResizeHandle::Left => (x0, cy),
        };

        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(size), Px(size)))
    }

    pub(super) fn screen_to_canvas(
        bounds: Rect,
        screen: Point,
        pan: CanvasPoint,
        zoom: f32,
    ) -> CanvasPoint {
        let viewport = Self::viewport_from_pan_zoom(bounds, pan, zoom);
        let p = viewport.screen_to_canvas(screen);
        CanvasPoint { x: p.x.0, y: p.y.0 }
    }

    pub(super) fn clamp_pan_to_translate_extent(
        pan: CanvasPoint,
        zoom: f32,
        bounds: Rect,
        extent: crate::core::CanvasRect,
    ) -> CanvasPoint {
        let extent_rect = Rect::new(
            Point::new(Px(extent.origin.x), Px(extent.origin.y)),
            Size::new(Px(extent.size.width), Px(extent.size.height)),
        );

        let view = clamp_pan_zoom_view(
            bounds,
            PanZoom2D {
                pan: Point::new(Px(pan.x), Px(pan.y)),
                zoom,
            },
            PanZoomConstraints2D {
                min_zoom: zoom,
                max_zoom: zoom,
                translate_extent_canvas: Some(extent_rect),
            },
        );

        CanvasPoint {
            x: view.pan.x.0,
            y: view.pan.y.0,
        }
    }

    pub(super) fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
        fn snap_axis(value: f32, grid: f32) -> f32 {
            if !value.is_finite() {
                return value;
            }
            if !grid.is_finite() || grid <= 0.0 {
                return value;
            }
            (value / grid).round() * grid
        }

        CanvasPoint {
            x: snap_axis(pos.x, grid.width),
            y: snap_axis(pos.y, grid.height),
        }
    }
}
