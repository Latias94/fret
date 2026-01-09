use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::CanvasSize;

use super::super::geometry::{node_ports, node_size_default_px};
use super::super::state::{NodeResizeHandle, ViewSnapshot};
use super::NodeGraphCanvas;

fn clamp_finite_positive(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        fallback.max(0.0)
    }
}

fn resolve_min_size_px<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    host: &mut H,
    node: crate::core::NodeId,
    fallback: (f32, f32),
) -> CanvasSize {
    let (mut w, mut h) = fallback;

    let style = canvas.style.clone();
    let _ = canvas.graph.read_ref(host, |graph| {
        let (inputs, outputs) = node_ports(graph, node);
        let (mw, mh) = node_size_default_px(inputs.len(), outputs.len(), &style);
        w = w.max(mw);
        h = h.max(mh);
    });

    CanvasSize {
        width: clamp_finite_positive(w, 0.0),
        height: clamp_finite_positive(h, 0.0),
    }
}

fn size_canvas_to_px(size_canvas: (f32, f32), zoom: f32) -> CanvasSize {
    let z = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    CanvasSize {
        width: clamp_finite_positive(size_canvas.0 * z, 0.0),
        height: clamp_finite_positive(size_canvas.1 * z, 0.0),
    }
}

fn apply_resize_handle(
    handle: NodeResizeHandle,
    start_node_pos: crate::core::CanvasPoint,
    start_size_px: CanvasSize,
    start_pointer: Point,
    pointer: Point,
    zoom: f32,
    min_size_px: CanvasSize,
    max_size_px: Option<CanvasSize>,
    max_bounds_canvas: Option<crate::core::CanvasRect>,
    snap_grid: Option<crate::core::CanvasSize>,
) -> (crate::core::CanvasPoint, CanvasSize) {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };

    let start_w_canvas = start_size_px.width / zoom;
    let start_h_canvas = start_size_px.height / zoom;
    let start_left = start_node_pos.x;
    let start_top = start_node_pos.y;
    let start_right = start_left + start_w_canvas;
    let start_bottom = start_top + start_h_canvas;

    let dx = pointer.x.0 - start_pointer.x.0;
    let dy = pointer.y.0 - start_pointer.y.0;

    let mut left = start_left;
    let mut top = start_top;
    let mut right = start_right;
    let mut bottom = start_bottom;

    if handle.affects_left() {
        left = start_left + dx;
    }
    if handle.affects_right() {
        right = start_right + dx;
    }
    if handle.affects_top() {
        top = start_top + dy;
    }
    if handle.affects_bottom() {
        bottom = start_bottom + dy;
    }

    if let Some(grid) = snap_grid {
        let gx = grid.width.max(0.0);
        let gy = grid.height.max(0.0);
        let snap = |v: f32, g: f32| -> f32 {
            if !v.is_finite() || !g.is_finite() || g <= 1.0e-6 {
                return v;
            }
            (v / g).round() * g
        };

        if handle.affects_left() {
            left = snap(left, gx);
        }
        if handle.affects_right() {
            right = snap(right, gx);
        }
        if handle.affects_top() {
            top = snap(top, gy);
        }
        if handle.affects_bottom() {
            bottom = snap(bottom, gy);
        }
    }

    // Enforce minimum size (in canvas units).
    let min_w_canvas = min_size_px.width / zoom;
    let min_h_canvas = min_size_px.height / zoom;

    let mut w = (right - left).max(0.0);
    let mut h = (bottom - top).max(0.0);
    if w.is_finite() && w < min_w_canvas {
        w = min_w_canvas;
        if handle.affects_left() && !handle.affects_right() {
            left = right - w;
        } else {
            right = left + w;
        }
    }
    if h.is_finite() && h < min_h_canvas {
        h = min_h_canvas;
        if handle.affects_top() && !handle.affects_bottom() {
            top = bottom - h;
        } else {
            bottom = top + h;
        }
    }

    // Enforce maximum size (in canvas units) if present.
    if let Some(max_size_px) = max_size_px {
        let max_w_canvas = (max_size_px.width / zoom).max(min_w_canvas);
        let max_h_canvas = (max_size_px.height / zoom).max(min_h_canvas);

        if w.is_finite() && w > max_w_canvas {
            w = max_w_canvas;
            if handle.affects_left() && !handle.affects_right() {
                left = right - w;
            } else {
                right = left + w;
            }
        }

        if h.is_finite() && h > max_h_canvas {
            h = max_h_canvas;
            if handle.affects_top() && !handle.affects_bottom() {
                top = bottom - h;
            } else {
                bottom = top + h;
            }
        }
    }

    // Enforce max bounds (in canvas units) if present.
    if let Some(extent) = max_bounds_canvas {
        let min_x = extent.origin.x;
        let min_y = extent.origin.y;
        let max_x = extent.origin.x + extent.size.width;
        let max_y = extent.origin.y + extent.size.height;

        if handle.affects_left() && !handle.affects_right() {
            left = left.max(min_x);
            right = right.min(max_x);
            w = (right - left).max(min_w_canvas);
            left = right - w;
        } else if handle.affects_right() {
            right = right.min(max_x);
            w = (right - left).max(min_w_canvas);
            right = left + w;
        } else {
            // no horizontal resize, keep within by clamping origin.
            let w0 = (right - left).max(min_w_canvas);
            left = left.clamp(min_x, (max_x - w0).max(min_x));
            right = left + w0;
        }

        if handle.affects_top() && !handle.affects_bottom() {
            top = top.max(min_y);
            bottom = bottom.min(max_y);
            h = (bottom - top).max(min_h_canvas);
            top = bottom - h;
        } else if handle.affects_bottom() {
            bottom = bottom.min(max_y);
            h = (bottom - top).max(min_h_canvas);
            bottom = top + h;
        } else {
            let h0 = (bottom - top).max(min_h_canvas);
            top = top.clamp(min_y, (max_y - h0).max(min_y));
            bottom = top + h0;
        }
    }

    let new_pos = crate::core::CanvasPoint { x: left, y: top };
    let new_size_px = size_canvas_to_px((right - left, bottom - top), zoom);
    (new_pos, new_size_px)
}

pub(super) fn handle_node_resize_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    _modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(resize) = canvas.interaction.node_resize.clone() else {
        return false;
    };

    let constraints = canvas
        .graph
        .read_ref(cx.app, |graph| {
            canvas
                .presenter
                .node_resize_constraints_px(graph, resize.node, &canvas.style)
                .normalized()
        })
        .ok()
        .unwrap_or_default();

    let min_size_px = resolve_min_size_px(
        canvas,
        cx.app,
        resize.node,
        constraints.min_size_px.unwrap_or((0.0, 0.0)),
    );
    let max_size_px = constraints.max_size_px.map(|(w, h)| CanvasSize {
        width: clamp_finite_positive(w, 0.0),
        height: clamp_finite_positive(h, 0.0),
    });

    let max_bounds_canvas = canvas
        .graph
        .read_ref(cx.app, |g| {
            let mut bound = snapshot.interaction.node_extent;
            if let Some(node) = g.nodes.get(&resize.node)
                && let Some(parent) = node.parent
                && let Some(group) = g.groups.get(&parent)
            {
                // Prefer the tighter bound (group bounds).
                let group_rect = group.rect;
                bound = Some(match bound {
                    Some(extent) => crate::core::CanvasRect {
                        origin: crate::core::CanvasPoint {
                            x: extent.origin.x.max(group_rect.origin.x),
                            y: extent.origin.y.max(group_rect.origin.y),
                        },
                        size: crate::core::CanvasSize {
                            width: (extent.origin.x + extent.size.width)
                                .min(group_rect.origin.x + group_rect.size.width)
                                - extent.origin.x.max(group_rect.origin.x),
                            height: (extent.origin.y + extent.size.height)
                                .min(group_rect.origin.y + group_rect.size.height)
                                - extent.origin.y.max(group_rect.origin.y),
                        },
                    },
                    None => group_rect,
                });
            }
            bound
        })
        .ok()
        .flatten()
        .map(|mut extent| {
            if extent.size.width.is_finite() {
                extent.size.width = extent.size.width.max(0.0);
            } else {
                extent.size.width = 0.0;
            }
            if extent.size.height.is_finite() {
                extent.size.height = extent.size.height.max(0.0);
            } else {
                extent.size.height = 0.0;
            }
            extent
        });

    let (new_pos, new_size_px) = apply_resize_handle(
        resize.handle,
        resize.start_node_pos,
        resize.start_size,
        resize.start_pos,
        position,
        zoom,
        min_size_px,
        max_size_px,
        max_bounds_canvas,
        snapshot
            .interaction
            .snap_to_grid
            .then_some(snapshot.interaction.snap_grid),
    );

    let _ = canvas.graph.update(cx.app, |g, _cx| {
        let Some(node) = g.nodes.get_mut(&resize.node) else {
            return;
        };
        node.pos = new_pos;
        node.size = Some(new_size_px);
    });

    // Invalidate derived geometry caches that depend on node bounds.
    canvas.geometry.key = None;
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

#[cfg(test)]
mod tests {
    use super::{NodeResizeHandle, apply_resize_handle};
    use crate::core::{CanvasPoint, CanvasRect, CanvasSize};
    use fret_core::{Point, Px};

    #[test]
    fn resize_right_increases_width_and_keeps_origin() {
        let start_pos = CanvasPoint { x: 10.0, y: 20.0 };
        let start_size_px = CanvasSize {
            width: 100.0,
            height: 50.0,
        };
        let start_pointer = Point::new(Px(0.0), Px(0.0));
        let pointer = Point::new(Px(10.0), Px(0.0)); // dx=10 canvas
        let zoom = 1.0;
        let min = CanvasSize {
            width: 10.0,
            height: 10.0,
        };

        let (pos, size) = apply_resize_handle(
            NodeResizeHandle::Right,
            start_pos,
            start_size_px,
            start_pointer,
            pointer,
            zoom,
            min,
            None,
            None,
            None,
        );
        assert_eq!(pos, start_pos);
        assert_eq!(size.width, 110.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn resize_left_moves_origin_and_keeps_right_edge() {
        let start_pos = CanvasPoint { x: 10.0, y: 20.0 };
        let start_size_px = CanvasSize {
            width: 100.0,
            height: 50.0,
        };
        let start_pointer = Point::new(Px(0.0), Px(0.0));
        let pointer = Point::new(Px(10.0), Px(0.0)); // dx=10 canvas
        let zoom = 1.0;
        let min = CanvasSize {
            width: 10.0,
            height: 10.0,
        };

        let (pos, size) = apply_resize_handle(
            NodeResizeHandle::Left,
            start_pos,
            start_size_px,
            start_pointer,
            pointer,
            zoom,
            min,
            None,
            None,
            None,
        );
        assert_eq!(pos.x, 20.0);
        assert_eq!(pos.y, 20.0);
        assert_eq!(size.width, 90.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn resize_respects_node_extent_bounds() {
        let start_pos = CanvasPoint { x: 0.0, y: 0.0 };
        let start_size_px = CanvasSize {
            width: 100.0,
            height: 50.0,
        };
        let start_pointer = Point::new(Px(0.0), Px(0.0));
        let pointer = Point::new(Px(200.0), Px(0.0)); // attempt to grow a lot
        let zoom = 1.0;
        let min = CanvasSize {
            width: 10.0,
            height: 10.0,
        };
        let extent = CanvasRect {
            origin: CanvasPoint { x: 0.0, y: 0.0 },
            size: CanvasSize {
                width: 120.0,
                height: 120.0,
            },
        };

        let (_pos, size) = apply_resize_handle(
            NodeResizeHandle::Right,
            start_pos,
            start_size_px,
            start_pointer,
            pointer,
            zoom,
            min,
            None,
            Some(extent),
            None,
        );
        assert_eq!(size.width, 120.0);
    }

    #[test]
    fn resize_snaps_moved_edge_to_grid_when_enabled() {
        let start_pos = CanvasPoint { x: 0.0, y: 0.0 };
        let start_size_px = CanvasSize {
            width: 100.0,
            height: 50.0,
        };
        let start_pointer = Point::new(Px(0.0), Px(0.0));
        let pointer = Point::new(Px(7.0), Px(0.0)); // dx=7 canvas
        let zoom = 1.0;
        let min = CanvasSize {
            width: 10.0,
            height: 10.0,
        };
        let grid = CanvasSize {
            width: 10.0,
            height: 10.0,
        };

        let (_pos, size) = apply_resize_handle(
            NodeResizeHandle::Right,
            start_pos,
            start_size_px,
            start_pointer,
            pointer,
            zoom,
            min,
            None,
            None,
            Some(grid),
        );
        assert_eq!(size.width, 110.0);
    }

    #[test]
    fn resize_respects_max_size_constraints() {
        let start_pos = CanvasPoint { x: 0.0, y: 0.0 };
        let start_size_px = CanvasSize {
            width: 100.0,
            height: 50.0,
        };
        let start_pointer = Point::new(Px(0.0), Px(0.0));
        let pointer = Point::new(Px(200.0), Px(200.0)); // attempt to grow a lot
        let zoom = 1.0;
        let min = CanvasSize {
            width: 10.0,
            height: 10.0,
        };
        let max = CanvasSize {
            width: 120.0,
            height: 80.0,
        };

        let (_pos, size) = apply_resize_handle(
            NodeResizeHandle::BottomRight,
            start_pos,
            start_size_px,
            start_pointer,
            pointer,
            zoom,
            min,
            Some(max),
            None,
            None,
        );
        assert_eq!(size.width, 120.0);
        assert_eq!(size.height, 80.0);
    }
}
