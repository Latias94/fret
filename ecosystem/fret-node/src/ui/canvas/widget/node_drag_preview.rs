use std::collections::HashMap;

use fret_ui::UiHost;

use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, GroupId, NodeExtent, NodeId as GraphNodeId,
};
use crate::ui::canvas::geometry::node_rect_origin_from_anchor;
use crate::ui::canvas::state::NodeDrag;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn compute_preview_positions<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    delta: CanvasPoint,
    multi_drag: bool,
) -> (Vec<(GraphNodeId, CanvasPoint)>, Vec<(GroupId, CanvasRect)>) {
    let geometry = canvas.canvas_geometry(&*cx.app, snapshot);
    let node_origin = snapshot.interaction.node_origin.normalized();
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut next_nodes: Vec<(GraphNodeId, CanvasPoint)> =
                Vec::with_capacity(drag.nodes.len());
            let mut group_overrides: HashMap<GroupId, CanvasRect> = HashMap::new();

            for (id, start) in &drag.nodes {
                let Some(node) = graph.nodes.get(id) else {
                    continue;
                };

                let mut target = CanvasPoint {
                    x: start.x + delta.x,
                    y: start.y + delta.y,
                };

                let Some(node_geom) = geometry.nodes.get(id) else {
                    continue;
                };
                let node_size = CanvasSize {
                    width: node_geom.rect.size.width.0,
                    height: node_geom.rect.size.height.0,
                };

                if !multi_drag && let Some(extent) = snapshot.interaction.node_extent {
                    target = super::node_drag_constraints::clamp_anchor_in_rect_with_size(
                        target,
                        node_size,
                        extent,
                        node_origin,
                    );
                }

                if let Some(NodeExtent::Rect { rect }) = node.extent {
                    target = super::node_drag_constraints::clamp_anchor_in_rect_with_size(
                        target,
                        node_size,
                        rect,
                        node_origin,
                    );
                }

                let expand_parent = node.expand_parent.unwrap_or(false);
                if let Some(parent) = node.parent {
                    let parent_rect = graph.groups.get(&parent).map(|group| group.rect);
                    let clamp_to_parent = !expand_parent
                        && match node.extent {
                            Some(NodeExtent::Parent) | None | Some(NodeExtent::Rect { .. }) => true,
                        };

                    if clamp_to_parent && let Some(group_rect) = parent_rect {
                        target = super::node_drag_constraints::clamp_anchor_in_rect_with_size(
                            target,
                            node_size,
                            group_rect,
                            node_origin,
                        );
                    } else if expand_parent && let Some(group_rect) = parent_rect {
                        let rect_origin =
                            node_rect_origin_from_anchor(target, node_size, node_origin);
                        let child_rect = CanvasRect {
                            origin: rect_origin,
                            size: node_size,
                        };
                        let next = super::node_drag_constraints::union_rect(group_rect, child_rect);
                        group_overrides
                            .entry(parent)
                            .and_modify(|rect| {
                                *rect = super::node_drag_constraints::union_rect(*rect, next)
                            })
                            .or_insert(next);
                    }
                }

                next_nodes.push((*id, target));
            }

            let mut next_groups: Vec<(GroupId, CanvasRect)> = group_overrides.into_iter().collect();
            next_groups.sort_by(|a, b| a.0.cmp(&b.0));
            (next_nodes, next_groups)
        })
        .ok()
        .unwrap_or_default()
}

pub(super) fn update_drag_preview_state(
    drag: &mut NodeDrag,
    next_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    next_groups: Vec<(GroupId, CanvasRect)>,
) {
    if drag.current_nodes != next_nodes || drag.current_groups != next_groups {
        drag.current_nodes = next_nodes;
        drag.current_groups = next_groups;
        drag.preview_rev = drag.preview_rev.wrapping_add(1);
    }
}
