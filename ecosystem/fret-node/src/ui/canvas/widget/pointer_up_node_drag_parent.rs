use std::collections::{BTreeMap, HashMap};

use fret_core::Rect;
use fret_ui::UiHost;

use crate::core::{GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::state::{NodeDrag, ViewSnapshot};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn parent_changes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
) -> Vec<(GraphNodeId, Option<GroupId>, Option<GroupId>)> {
    let geom = canvas.canvas_geometry(&*host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut changes: Vec<(GraphNodeId, Option<GroupId>, Option<GroupId>)> = Vec::new();

            for (node_id, _start) in &drag.nodes {
                let Some(node) = graph.nodes.get(node_id) else {
                    continue;
                };
                let Some(node_geom) = geom.nodes.get(node_id) else {
                    continue;
                };
                let Some(pos) = end_positions.get(node_id).copied() else {
                    continue;
                };

                let rect = Rect::new(
                    fret_core::Point::new(fret_core::Px(pos.x), fret_core::Px(pos.y)),
                    node_geom.rect.size,
                );
                let new_parent = best_parent_group(rect, graph, group_overrides);
                if node.parent != new_parent {
                    changes.push((*node_id, node.parent, new_parent));
                }
            }

            changes
        })
        .ok()
        .unwrap_or_default()
}

fn best_parent_group(
    rect: Rect,
    graph: &crate::core::Graph,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
) -> Option<GroupId> {
    let rect_min_x = rect.origin.x.0;
    let rect_min_y = rect.origin.y.0;
    let rect_max_x = rect.origin.x.0 + rect.size.width.0;
    let rect_max_y = rect.origin.y.0 + rect.size.height.0;

    let mut best: Option<(GroupId, f32)> = None;
    for (group_id, group) in &graph.groups {
        let group_rect = group_overrides.get(group_id).copied().unwrap_or(group.rect);
        let gx0 = group_rect.origin.x;
        let gy0 = group_rect.origin.y;
        let gx1 = group_rect.origin.x + group_rect.size.width;
        let gy1 = group_rect.origin.y + group_rect.size.height;
        if rect_min_x >= gx0 && rect_min_y >= gy0 && rect_max_x <= gx1 && rect_max_y <= gy1 {
            let area = (group_rect.size.width.max(0.0)) * (group_rect.size.height.max(0.0));
            match best {
                Some((_id, best_area)) if best_area <= area => {}
                _ => best = Some((*group_id, area)),
            }
        }
    }

    best.map(|(id, _)| id)
}
