use crate::core::{CanvasPoint, CanvasRect, CanvasSize};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::state::NodeResizeHandle;

use fret_core::Point;

use super::clamp_finite_positive;

#[derive(Clone, Copy)]
struct ResizeEdges {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl ResizeEdges {
    fn width(self) -> f32 {
        (self.right - self.left).max(0.0)
    }

    fn height(self) -> f32 {
        (self.bottom - self.top).max(0.0)
    }
}

pub(in super::super) fn apply_resize_handle(
    handle: NodeResizeHandle,
    keep_aspect_ratio: bool,
    start_node_pos: CanvasPoint,
    node_origin: NodeGraphNodeOrigin,
    start_size_px: CanvasSize,
    start_pointer: Point,
    pointer: Point,
    zoom: f32,
    min_size_px: CanvasSize,
    max_size_px: Option<CanvasSize>,
    max_bounds_canvas: Option<CanvasRect>,
    snap_grid: Option<CanvasSize>,
) -> (CanvasPoint, CanvasSize) {
    let zoom = sanitize_zoom(zoom);
    let node_origin = node_origin.normalized();
    let start_w_canvas = start_size_px.width / zoom;
    let start_h_canvas = start_size_px.height / zoom;
    let mut edges = start_edges(start_node_pos, node_origin, start_w_canvas, start_h_canvas);

    let dx = pointer.x.0 - start_pointer.x.0;
    let dy = pointer.y.0 - start_pointer.y.0;

    apply_pointer_delta(handle, &mut edges, dx, dy);
    snap_edges_to_grid(handle, &mut edges, snap_grid);
    apply_aspect_ratio(
        handle,
        keep_aspect_ratio,
        start_w_canvas,
        start_h_canvas,
        dx,
        dy,
        &mut edges,
    );

    let min_w_canvas = min_size_px.width / zoom;
    let min_h_canvas = min_size_px.height / zoom;
    enforce_min_size(handle, min_w_canvas, min_h_canvas, &mut edges);
    if let Some(max_size_px) = max_size_px {
        enforce_max_size(
            handle,
            min_w_canvas,
            min_h_canvas,
            max_size_px,
            zoom,
            &mut edges,
        );
    }
    if let Some(max_bounds_canvas) = max_bounds_canvas {
        enforce_max_bounds(
            handle,
            max_bounds_canvas,
            min_w_canvas,
            min_h_canvas,
            &mut edges,
        );
    }

    let new_size_px = size_canvas_to_px((edges.width(), edges.height()), zoom);
    let w_canvas = (new_size_px.width / zoom).max(0.0);
    let h_canvas = (new_size_px.height / zoom).max(0.0);
    let anchor = CanvasPoint {
        x: edges.left + node_origin.x * w_canvas,
        y: edges.top + node_origin.y * h_canvas,
    };
    (anchor, new_size_px)
}

fn sanitize_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    }
}

fn start_edges(
    start_node_pos: CanvasPoint,
    node_origin: NodeGraphNodeOrigin,
    start_w_canvas: f32,
    start_h_canvas: f32,
) -> ResizeEdges {
    let left = start_node_pos.x - node_origin.x * start_w_canvas;
    let top = start_node_pos.y - node_origin.y * start_h_canvas;
    ResizeEdges {
        left,
        top,
        right: left + start_w_canvas,
        bottom: top + start_h_canvas,
    }
}

fn apply_pointer_delta(handle: NodeResizeHandle, edges: &mut ResizeEdges, dx: f32, dy: f32) {
    if handle.affects_left() {
        edges.left += dx;
    }
    if handle.affects_right() {
        edges.right += dx;
    }
    if handle.affects_top() {
        edges.top += dy;
    }
    if handle.affects_bottom() {
        edges.bottom += dy;
    }
}

fn snap_edges_to_grid(
    handle: NodeResizeHandle,
    edges: &mut ResizeEdges,
    snap_grid: Option<CanvasSize>,
) {
    let Some(grid) = snap_grid else {
        return;
    };
    let gx = grid.width.max(0.0);
    let gy = grid.height.max(0.0);

    if handle.affects_left() {
        edges.left = snap_axis(edges.left, gx);
    }
    if handle.affects_right() {
        edges.right = snap_axis(edges.right, gx);
    }
    if handle.affects_top() {
        edges.top = snap_axis(edges.top, gy);
    }
    if handle.affects_bottom() {
        edges.bottom = snap_axis(edges.bottom, gy);
    }
}

fn snap_axis(v: f32, g: f32) -> f32 {
    if !v.is_finite() || !g.is_finite() || g <= 1.0e-6 {
        return v;
    }
    (v / g).round() * g
}

fn apply_aspect_ratio(
    handle: NodeResizeHandle,
    keep_aspect_ratio: bool,
    start_w_canvas: f32,
    start_h_canvas: f32,
    dx: f32,
    dy: f32,
    edges: &mut ResizeEdges,
) {
    let keep_aspect_ratio = keep_aspect_ratio
        && (handle.affects_left() || handle.affects_right())
        && (handle.affects_top() || handle.affects_bottom());
    if !keep_aspect_ratio || !start_h_canvas.is_finite() || start_h_canvas <= 1.0e-6 {
        return;
    }

    let aspect_ratio = start_w_canvas / start_h_canvas;
    if !aspect_ratio.is_finite() || aspect_ratio <= 1.0e-6 {
        return;
    }

    let width_drives = dx.abs() >= dy.abs();
    if width_drives {
        let h = (edges.width() / aspect_ratio).max(0.0);
        if handle.affects_top() && !handle.affects_bottom() {
            edges.top = edges.bottom - h;
        } else {
            edges.bottom = edges.top + h;
        }
    } else {
        let w = (edges.height() * aspect_ratio).max(0.0);
        if handle.affects_left() && !handle.affects_right() {
            edges.left = edges.right - w;
        } else {
            edges.right = edges.left + w;
        }
    }
}

fn enforce_min_size(
    handle: NodeResizeHandle,
    min_w_canvas: f32,
    min_h_canvas: f32,
    edges: &mut ResizeEdges,
) {
    let mut w = edges.width();
    let mut h = edges.height();

    if w.is_finite() && w < min_w_canvas {
        w = min_w_canvas;
        set_width(handle, edges, w);
    }
    if h.is_finite() && h < min_h_canvas {
        h = min_h_canvas;
        set_height(handle, edges, h);
    }
}

fn enforce_max_size(
    handle: NodeResizeHandle,
    min_w_canvas: f32,
    min_h_canvas: f32,
    max_size_px: CanvasSize,
    zoom: f32,
    edges: &mut ResizeEdges,
) {
    let max_w_canvas = (max_size_px.width / zoom).max(min_w_canvas);
    let max_h_canvas = (max_size_px.height / zoom).max(min_h_canvas);

    let mut w = edges.width();
    let mut h = edges.height();
    if w.is_finite() && w > max_w_canvas {
        w = max_w_canvas;
        set_width(handle, edges, w);
    }
    if h.is_finite() && h > max_h_canvas {
        h = max_h_canvas;
        set_height(handle, edges, h);
    }
}

fn enforce_max_bounds(
    handle: NodeResizeHandle,
    extent: CanvasRect,
    min_w_canvas: f32,
    min_h_canvas: f32,
    edges: &mut ResizeEdges,
) {
    let min_x = extent.origin.x;
    let min_y = extent.origin.y;
    let max_x = extent.origin.x + extent.size.width;
    let max_y = extent.origin.y + extent.size.height;

    if handle.affects_left() && !handle.affects_right() {
        edges.left = edges.left.max(min_x);
        edges.right = edges.right.min(max_x);
        let w = edges.width().max(min_w_canvas);
        edges.left = edges.right - w;
    } else if handle.affects_right() {
        edges.right = edges.right.min(max_x);
        let w = edges.width().max(min_w_canvas);
        edges.right = edges.left + w;
    } else {
        let w = edges.width().max(min_w_canvas);
        edges.left = edges.left.clamp(min_x, (max_x - w).max(min_x));
        edges.right = edges.left + w;
    }

    if handle.affects_top() && !handle.affects_bottom() {
        edges.top = edges.top.max(min_y);
        edges.bottom = edges.bottom.min(max_y);
        let h = edges.height().max(min_h_canvas);
        edges.top = edges.bottom - h;
    } else if handle.affects_bottom() {
        edges.bottom = edges.bottom.min(max_y);
        let h = edges.height().max(min_h_canvas);
        edges.bottom = edges.top + h;
    } else {
        let h = edges.height().max(min_h_canvas);
        edges.top = edges.top.clamp(min_y, (max_y - h).max(min_y));
        edges.bottom = edges.top + h;
    }
}

fn set_width(handle: NodeResizeHandle, edges: &mut ResizeEdges, width: f32) {
    if handle.affects_left() && !handle.affects_right() {
        edges.left = edges.right - width;
    } else {
        edges.right = edges.left + width;
    }
}

fn set_height(handle: NodeResizeHandle, edges: &mut ResizeEdges, height: f32) {
    if handle.affects_top() && !handle.affects_bottom() {
        edges.top = edges.bottom - height;
    } else {
        edges.bottom = edges.top + height;
    }
}

fn size_canvas_to_px(size_canvas: (f32, f32), zoom: f32) -> CanvasSize {
    let z = sanitize_zoom(zoom);
    CanvasSize {
        width: clamp_finite_positive(size_canvas.0 * z, 0.0),
        height: clamp_finite_positive(size_canvas.1 * z, 0.0),
    }
}
