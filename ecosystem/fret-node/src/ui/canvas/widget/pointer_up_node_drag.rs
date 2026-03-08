use std::collections::{BTreeMap, HashMap};

use fret_core::Rect;
use fret_ui::UiHost;

use crate::core::GroupId;
use crate::ops::GraphOp;
use crate::runtime::callbacks::NodeDragEndOutcome;
use crate::ui::canvas::state::ViewSnapshot;

use super::pointer_up_finish::finish_pointer_up;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_node_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let Some(drag) = canvas.interaction.node_drag.take() else {
        return false;
    };

    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    let end_positions: HashMap<crate::core::NodeId, crate::core::CanvasPoint> = drag
        .current_nodes
        .iter()
        .copied()
        .collect::<HashMap<_, _>>();
    let group_overrides: BTreeMap<GroupId, crate::core::CanvasRect> = drag
        .current_groups
        .iter()
        .copied()
        .collect::<BTreeMap<_, _>>();

    let parent_changes: Vec<(crate::core::NodeId, Option<GroupId>, Option<GroupId>)> = canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut changes: Vec<(crate::core::NodeId, Option<GroupId>, Option<GroupId>)> =
                Vec::new();

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
                    if rect_min_x >= gx0
                        && rect_min_y >= gy0
                        && rect_max_x <= gx1
                        && rect_max_y <= gy1
                    {
                        let area =
                            (group_rect.size.width.max(0.0)) * (group_rect.size.height.max(0.0));
                        match best {
                            Some((_id, best_area)) if best_area <= area => {}
                            _ => best = Some((*group_id, area)),
                        }
                    }
                }

                let new_parent = best.map(|(id, _)| id);
                if node.parent != new_parent {
                    changes.push((*node_id, node.parent, new_parent));
                }
            }

            changes
        })
        .ok()
        .unwrap_or_default();

    let mut ops: Vec<GraphOp> = Vec::new();
    for (id, start) in &drag.nodes {
        let Some(end) = end_positions.get(id).copied() else {
            continue;
        };
        if end != *start {
            ops.push(GraphOp::SetNodePos {
                id: *id,
                from: *start,
                to: end,
            });
        }
    }

    for (node_id, from, to) in &parent_changes {
        ops.push(GraphOp::SetNodeParent {
            id: *node_id,
            from: *from,
            to: *to,
        });
    }

    let group_rect_ops: Vec<GraphOp> = canvas
        .graph
        .read_ref(cx.app, |graph| {
            group_overrides
                .iter()
                .filter_map(|(&id, &to)| {
                    let from = graph.groups.get(&id).map(|g| g.rect)?;
                    (from != to).then_some(GraphOp::SetGroupRect { id, from, to })
                })
                .collect()
        })
        .ok()
        .unwrap_or_default();
    ops.extend(group_rect_ops);

    let drag_outcome = if ops.is_empty() {
        NodeDragEndOutcome::NoOp
    } else {
        let label = if ops
            .iter()
            .all(|op| matches!(op, GraphOp::SetNodeParent { .. }))
        {
            if ops.len() == 1 {
                "Set Node Parent"
            } else {
                "Set Node Parents"
            }
        } else if ops.len() == 1 {
            "Move Node"
        } else {
            "Move Nodes"
        };
        if canvas.commit_ops(cx.app, cx.window, Some(label), ops) {
            NodeDragEndOutcome::Committed
        } else {
            NodeDragEndOutcome::Rejected
        }
    };

    canvas.emit_node_drag_end(drag.primary, &drag.node_ids, drag_outcome);
    canvas.interaction.pending_node_drag = None;
    canvas.interaction.snap_guides = None;
    finish_pointer_up(cx);
    true
}
