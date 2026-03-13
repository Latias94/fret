use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn viewport_from_pan_zoom(
        bounds: Rect,
        pan: CanvasPoint,
        zoom: f32,
    ) -> CanvasViewport2D {
        super::view_math_viewport::viewport_from_pan_zoom(bounds, pan, zoom)
    }

    pub(super) fn viewport_from_snapshot(
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> CanvasViewport2D {
        super::view_math_viewport::viewport_from_snapshot(bounds, snapshot)
    }

    pub(super) fn close_button_rect(pan: CanvasPoint, zoom: f32) -> Rect {
        super::view_math_rect::close_button_rect(pan, zoom)
    }

    pub(super) fn rect_contains(rect: Rect, pos: Point) -> bool {
        super::view_math_rect::rect_contains(rect, pos)
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
        super::view_math_rect::node_resize_handle_rect(
            node_rect,
            handle,
            zoom,
            self.style.geometry.resize_handle_size,
        )
    }

    pub(super) fn screen_to_canvas(
        bounds: Rect,
        screen: Point,
        pan: CanvasPoint,
        zoom: f32,
    ) -> CanvasPoint {
        super::view_math_viewport::screen_to_canvas(bounds, screen, pan, zoom)
    }

    pub(super) fn clamp_pan_to_translate_extent(
        pan: CanvasPoint,
        zoom: f32,
        bounds: Rect,
        extent: crate::core::CanvasRect,
    ) -> CanvasPoint {
        super::view_math_viewport::clamp_pan_to_translate_extent(pan, zoom, bounds, extent)
    }

    pub(super) fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
        super::view_math_viewport::snap_canvas_point(pos, grid)
    }
}
