use std::collections::HashSet;

use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};
use crate::core::{CanvasRect, CanvasSize, NodeExtent};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

fn clamp_point_in_rect_with_size(
    pos: CanvasPoint,
    size: CanvasSize,
    extent: CanvasRect,
) -> CanvasPoint {
    let mut out = pos;
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

pub(super) fn handle_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(drag) = canvas.interaction.node_drag.clone() else {
        return false;
    };

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

            let snap = super::super::snaplines::snap_delta_for_rects(
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
    let _ = canvas.graph.update(cx.app, |g, _cx| {
        for (id, start) in &drag.nodes {
            let Some(node) = g.nodes.get(id) else {
                continue;
            };
            let from = node.pos;
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

            if let Some(extent) = snapshot.interaction.node_extent {
                to = clamp_point_in_rect_with_size(to, node_size, extent);
            }

            if let Some(NodeExtent::Rect { rect }) = node.extent {
                to = clamp_point_in_rect_with_size(to, node_size, rect);
            }

            let expand_parent = node.expand_parent.unwrap_or(false);
            if let Some(parent) = node.parent {
                let parent_rect = g.groups.get(&parent).map(|gr| gr.rect);

                let clamp_to_parent = !expand_parent
                    && match node.extent {
                        Some(NodeExtent::Parent) | None | Some(NodeExtent::Rect { .. }) => true,
                    };

                if clamp_to_parent && let Some(group_rect) = parent_rect {
                    to = clamp_point_in_rect_with_size(to, node_size, group_rect);
                } else if expand_parent && let Some(group) = g.groups.get_mut(&parent) {
                    let child_rect = CanvasRect {
                        origin: to,
                        size: node_size,
                    };
                    group.rect = union_rect(group.rect, child_rect);
                }
            }

            if from != to {
                if let Some(node) = g.nodes.get_mut(id) {
                    node.pos = to;
                }
            }
        }
    });

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
