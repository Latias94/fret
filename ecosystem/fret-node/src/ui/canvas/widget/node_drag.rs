use std::collections::{HashMap, HashSet};

use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};
use crate::core::{CanvasRect, CanvasSize, NodeExtent};
use crate::io::NodeGraphNodeOrigin;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};
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

fn clamp_anchor_in_rect_with_size(
    anchor: CanvasPoint,
    size: CanvasSize,
    extent: CanvasRect,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    let rect_origin = node_rect_origin_from_anchor(anchor, size, node_origin);
    let clamped = clamp_rect_origin_in_rect_with_size(rect_origin, size, extent);
    node_anchor_from_rect_origin(clamped, size, node_origin)
}

fn union_rect(a: CanvasRect, b: CanvasRect) -> CanvasRect {
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
    geom: &crate::ui::canvas::geometry::CanvasGeometry,
    nodes: &[GraphNodeId],
) -> Option<(CanvasPoint, CanvasSize)> {
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    let mut any = false;

    for id in nodes {
        let Some(node_geom) = geom.nodes.get(id) else {
            continue;
        };
        let w = node_geom.rect.size.width.0.max(0.0);
        let h = node_geom.rect.size.height.0.max(0.0);
        let x0 = node_geom.rect.origin.x.0;
        let y0 = node_geom.rect.origin.y.0;
        if !x0.is_finite() || !y0.is_finite() || !w.is_finite() || !h.is_finite() {
            continue;
        }

        any = true;
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x0 + w);
        max_y = max_y.max(y0 + h);
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

pub(super) fn handle_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(mut drag) = canvas.interaction.node_drag.clone() else {
        return false;
    };
    let multi_drag = drag.nodes.len() > 1;

    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();
    let snap_to_grid = snapshot.interaction.snap_to_grid;
    let snap_grid = snapshot.interaction.snap_grid;
    let snaplines = snapshot.interaction.snaplines;
    let snaplines_threshold_screen = snapshot.interaction.snaplines_threshold;

    let start_anchor = Point::new(
        Px(drag.start_pos.x.0 - drag.grab_offset.x.0),
        Px(drag.start_pos.y.0 - drag.grab_offset.y.0),
    );
    let new_anchor = Point::new(
        Px(position.x.0 - drag.grab_offset.x.0 - auto_pan_delta.x),
        Px(position.y.0 - drag.grab_offset.y.0 - auto_pan_delta.y),
    );
    let mut delta = CanvasPoint {
        x: new_anchor.x.0 - start_anchor.x.0,
        y: new_anchor.y.0 - start_anchor.y.0,
    };

    if snap_to_grid {
        let primary_start = drag
            .nodes
            .iter()
            .find(|(id, _)| *id == drag.primary)
            .map(|(_, p)| *p)
            .unwrap_or_default();
        let primary_target = CanvasPoint {
            x: primary_start.x + delta.x,
            y: primary_start.y + delta.y,
        };
        let snapped = NodeGraphCanvasWith::<M>::snap_canvas_point(primary_target, snap_grid);
        delta = CanvasPoint {
            x: snapped.x - primary_start.x,
            y: snapped.y - primary_start.y,
        };
    }

    if snaplines {
        let threshold_canvas = canvas_units_from_screen_px(snaplines_threshold_screen, zoom);

        let geom = canvas.canvas_geometry(&*cx.app, snapshot);
        let drag_nodes: HashSet<GraphNodeId> = drag.nodes.iter().map(|(id, _)| *id).collect();

        let mut group: Option<Rect> = None;
        for (id, start) in &drag.nodes {
            let Some(ng) = geom.nodes.get(id) else {
                continue;
            };
            let rect0 = Rect::new(Point::new(Px(start.x), Px(start.y)), ng.rect.size);
            group = Some(match group {
                Some(r) => super::rect_union(r, rect0),
                None => rect0,
            });
        }

        let mut candidates: Vec<Rect> = Vec::new();
        for (id, ng) in geom.nodes.iter() {
            if drag_nodes.contains(id) {
                continue;
            }
            candidates.push(ng.rect);
        }

        if let Some(group0) = group {
            let moving = Rect::new(
                Point::new(
                    Px(group0.origin.x.0 + delta.x),
                    Px(group0.origin.y.0 + delta.y),
                ),
                group0.size,
            );

            let snap = crate::ui::canvas::snaplines::snap_delta_for_rects(
                moving,
                &candidates,
                threshold_canvas,
            );
            canvas.interaction.snap_guides =
                (snap.guides.x.is_some() || snap.guides.y.is_some()).then_some(snap.guides);

            let allow_snap = !modifiers.alt && !modifiers.alt_gr;
            if allow_snap {
                delta.x += snap.delta_x;
                delta.y += snap.delta_y;
            }
        } else {
            canvas.interaction.snap_guides = None;
        }
    } else {
        canvas.interaction.snap_guides = None;
    }

    let geom_for_extent = canvas.canvas_geometry(&*cx.app, snapshot);
    let node_origin = snapshot.interaction.node_origin.normalized();
    if multi_drag && let Some(extent) = snapshot.interaction.node_extent {
        if let Some((group_min, group_size)) =
            dragged_group_bounds(&geom_for_extent, &drag.node_ids)
        {
            delta = clamp_delta_for_extent_rect(delta, group_min, group_size, extent);
        }
    }
    let (next_nodes, next_groups) = canvas
        .graph
        .read_ref(cx.app, |g| {
            let mut out_nodes: Vec<(GraphNodeId, CanvasPoint)> =
                Vec::with_capacity(drag.nodes.len());
            let mut group_overrides: HashMap<crate::core::GroupId, CanvasRect> = HashMap::new();

            for (id, start) in &drag.nodes {
                let Some(node) = g.nodes.get(id) else {
                    continue;
                };

                let mut to = CanvasPoint {
                    x: start.x + delta.x,
                    y: start.y + delta.y,
                };

                let Some(node_geom) = geom_for_extent.nodes.get(id) else {
                    continue;
                };
                let node_size = CanvasSize {
                    width: node_geom.rect.size.width.0,
                    height: node_geom.rect.size.height.0,
                };

                if !multi_drag && let Some(extent) = snapshot.interaction.node_extent {
                    to = clamp_anchor_in_rect_with_size(to, node_size, extent, node_origin);
                }

                if let Some(NodeExtent::Rect { rect }) = node.extent {
                    to = clamp_anchor_in_rect_with_size(to, node_size, rect, node_origin);
                }

                let expand_parent = node.expand_parent.unwrap_or(false);
                if let Some(parent) = node.parent {
                    let parent_rect = g.groups.get(&parent).map(|gr| gr.rect);

                    let clamp_to_parent = !expand_parent
                        && match node.extent {
                            Some(NodeExtent::Parent) | None | Some(NodeExtent::Rect { .. }) => true,
                        };

                    if clamp_to_parent && let Some(group_rect) = parent_rect {
                        to = clamp_anchor_in_rect_with_size(to, node_size, group_rect, node_origin);
                    } else if expand_parent && let Some(group_rect) = parent_rect {
                        let rect_origin = node_rect_origin_from_anchor(to, node_size, node_origin);
                        let child_rect = CanvasRect {
                            origin: rect_origin,
                            size: node_size,
                        };
                        let next = union_rect(group_rect, child_rect);
                        group_overrides
                            .entry(parent)
                            .and_modify(|r| *r = union_rect(*r, next))
                            .or_insert(next);
                    }
                }

                out_nodes.push((*id, to));
            }

            let mut out_groups: Vec<(crate::core::GroupId, CanvasRect)> =
                group_overrides.into_iter().collect();
            out_groups.sort_by(|a, b| a.0.cmp(&b.0));
            (out_nodes, out_groups)
        })
        .ok()
        .unwrap_or_default();

    if drag.current_nodes != next_nodes || drag.current_groups != next_groups {
        drag.current_nodes = next_nodes;
        drag.current_groups = next_groups;
        drag.preview_rev = drag.preview_rev.wrapping_add(1);
    }
    canvas.interaction.node_drag = Some(drag.clone());

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    canvas.emit_node_drag(drag.primary, &drag.node_ids);

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
