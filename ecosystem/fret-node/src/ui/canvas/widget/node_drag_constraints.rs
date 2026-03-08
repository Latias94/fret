use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect, CanvasSize, NodeId as GraphNodeId};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::geometry::{node_anchor_from_rect_origin, node_rect_origin_from_anchor};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

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

pub(super) fn apply_multi_drag_extent_delta<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    node_ids: &[GraphNodeId],
    delta: CanvasPoint,
    multi_drag: bool,
) -> CanvasPoint {
    if !multi_drag {
        return delta;
    }

    let Some(extent) = snapshot.interaction.node_extent else {
        return delta;
    };

    let geometry = canvas.canvas_geometry(&*cx.app, snapshot);
    let Some((group_min, group_size)) = dragged_group_bounds(&geometry, node_ids) else {
        return delta;
    };

    clamp_delta_for_extent_rect(delta, group_min, group_size, extent)
}

fn clamp_delta_for_extent_rect(
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

fn dragged_group_bounds(
    geometry: &crate::ui::canvas::geometry::CanvasGeometry,
    nodes: &[GraphNodeId],
) -> Option<(CanvasPoint, CanvasSize)> {
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    let mut any = false;

    for id in nodes {
        let Some(node_geom) = geometry.nodes.get(id) else {
            continue;
        };
        let width = node_geom.rect.size.width.0.max(0.0);
        let height = node_geom.rect.size.height.0.max(0.0);
        let x0 = node_geom.rect.origin.x.0;
        let y0 = node_geom.rect.origin.y.0;
        if !x0.is_finite() || !y0.is_finite() || !width.is_finite() || !height.is_finite() {
            continue;
        }

        any = true;
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x0 + width);
        max_y = max_y.max(y0 + height);
    }

    if !any || !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite()
    {
        return None;
    }

    Some((
        CanvasPoint { x: min_x, y: min_y },
        CanvasSize {
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        },
    ))
}
