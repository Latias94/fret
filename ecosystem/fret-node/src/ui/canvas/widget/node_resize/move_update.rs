use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::{CanvasRect, CanvasSize, NodeExtent};

use super::math;
use crate::ui::canvas::geometry::node_rect_origin_from_anchor;
use crate::ui::canvas::geometry::{node_ports, node_size_default_px};
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

fn resolve_min_size_px<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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
        width: math::clamp_finite_positive(w, 0.0),
        height: math::clamp_finite_positive(h, 0.0),
    }
}

pub(in super::super) fn handle_node_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(mut resize) = canvas.interaction.node_resize.clone() else {
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
        width: math::clamp_finite_positive(w, 0.0),
        height: math::clamp_finite_positive(h, 0.0),
    });

    let max_bounds_canvas = canvas
        .graph
        .read_ref(cx.app, |g| {
            let mut bound = snapshot.interaction.node_extent;
            let Some(node) = g.nodes.get(&resize.node) else {
                return bound;
            };

            if let Some(NodeExtent::Rect { rect }) = node.extent {
                bound = Some(match bound {
                    Some(b) => math::canvas_rect_intersection(b, rect),
                    None => rect,
                });
            }

            let expand_parent = node.expand_parent.unwrap_or(false);
            if !expand_parent
                && let Some(parent) = node.parent
                && let Some(group) = g.groups.get(&parent)
            {
                // Groups act as parent containers; by default child nodes are constrained within.
                // This matches XyFlow's `extent: 'parent'` behavior, with the escape hatch
                // `expand_parent=true` to avoid clamping and expand the parent instead.
                let group_rect = group.rect;
                bound = Some(match bound {
                    Some(b) => math::canvas_rect_intersection(b, group_rect),
                    None => group_rect,
                });
            }

            if node.extent == Some(NodeExtent::Parent) && !expand_parent && node.parent.is_none() {
                // No parent to clamp to.
            }

            bound
        })
        .ok()
        .flatten()
        .map(math::normalize_canvas_rect);

    let (new_pos, new_size_px) = math::apply_resize_handle(
        resize.handle,
        modifiers.shift,
        resize.start_node_pos,
        snapshot.interaction.node_origin,
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

    let current_size_opt = Some(new_size_px);
    let current_groups: Vec<(crate::core::GroupId, CanvasRect)> = canvas
        .graph
        .read_ref(cx.app, |g| {
            let Some(node) = g.nodes.get(&resize.node) else {
                return Vec::new();
            };
            let expand_parent = node.expand_parent.unwrap_or(false);
            let Some(parent) = node.parent else {
                return Vec::new();
            };
            if !expand_parent {
                return Vec::new();
            }
            let Some(group) = g.groups.get(&parent) else {
                return Vec::new();
            };
            let z = zoom.max(1.0e-6);
            let origin = snapshot.interaction.node_origin.normalized();
            let child_size_canvas = CanvasSize {
                width: (new_size_px.width / z).max(0.0),
                height: (new_size_px.height / z).max(0.0),
            };
            let child_rect = CanvasRect {
                origin: node_rect_origin_from_anchor(new_pos, child_size_canvas, origin),
                size: child_size_canvas,
            };
            vec![(parent, math::canvas_rect_union(group.rect, child_rect))]
        })
        .ok()
        .unwrap_or_default();

    if resize.current_node_pos != new_pos
        || resize.current_size_opt != current_size_opt
        || resize.current_groups != current_groups
    {
        resize.current_node_pos = new_pos;
        resize.current_size_opt = current_size_opt;
        resize.current_groups = current_groups;
        resize.preview_rev = resize.preview_rev.wrapping_add(1);
    }
    canvas.interaction.node_resize = Some(resize);

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
