use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use crate::core::GroupId;
use crate::ops::GraphOp;

use super::super::state::{PendingNodeSelectAction, ViewSnapshot};
use super::NodeGraphCanvas;

pub(super) fn handle_pointer_up<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    click_count: u8,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.last_canvas_pos = Some(NodeGraphCanvas::screen_to_canvas(
        cx.bounds,
        position,
        snapshot.pan,
        zoom,
    ));

    if button == MouseButton::Left
        && canvas.interaction.sticky_wire_ignore_next_up
        && canvas.interaction.wire_drag.is_some()
    {
        canvas.interaction.sticky_wire_ignore_next_up = false;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if button == MouseButton::Middle && canvas.interaction.panning {
        canvas.interaction.panning = false;
        canvas.stop_auto_pan_timer(cx.app);
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if button != MouseButton::Left {
        return false;
    }

    canvas.stop_auto_pan_timer(cx.app);

    if click_count == 2
        && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
        && let Some(edge_drag) = canvas.interaction.edge_drag.take()
    {
        canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_drag.edge, position);

        canvas.interaction.hover_edge = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if super::marquee::handle_left_up(canvas, cx) {
        return true;
    }

    if let Some(resize) = canvas.interaction.node_resize.take() {
        canvas.interaction.pending_node_resize = None;

        let end = canvas
            .graph
            .read_ref(cx.app, |g| g.nodes.get(&resize.node).and_then(|n| n.size))
            .ok()
            .flatten();

        if resize.start_size_opt != end {
            canvas.history.record(crate::ops::GraphTransaction {
                label: Some("Resize Node".to_string()),
                ops: vec![GraphOp::SetNodeSize {
                    id: resize.node,
                    from: resize.start_size_opt,
                    to: end,
                }],
            });
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(resize) = canvas.interaction.group_resize.take() {
        canvas.interaction.pending_group_resize = None;

        let end = canvas
            .graph
            .read_ref(cx.app, |g| g.groups.get(&resize.group).map(|gr| gr.rect))
            .ok()
            .flatten();

        if let Some(end) = end
            && end != resize.start_rect
        {
            canvas.history.record(crate::ops::GraphTransaction {
                label: Some("Resize Group".to_string()),
                ops: vec![GraphOp::SetGroupRect {
                    id: resize.group,
                    from: resize.start_rect,
                    to: end,
                }],
            });
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(drag) = canvas.interaction.group_drag.take() {
        canvas.interaction.pending_group_drag = None;

        let end_rect = canvas
            .graph
            .read_ref(cx.app, |g| g.groups.get(&drag.group).map(|gr| gr.rect))
            .ok()
            .flatten();

        let mut ops: Vec<GraphOp> = Vec::new();
        if let Some(end_rect) = end_rect
            && end_rect != drag.start_rect
        {
            ops.push(GraphOp::SetGroupRect {
                id: drag.group,
                from: drag.start_rect,
                to: end_rect,
            });
        }

        let mut node_ops = canvas
            .graph
            .read_ref(cx.app, |g| {
                drag.nodes
                    .iter()
                    .filter_map(|(id, start)| {
                        let end = g.nodes.get(id).map(|n| n.pos)?;
                        (end != *start).then_some(GraphOp::SetNodePos {
                            id: *id,
                            from: *start,
                            to: end,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .ok()
            .unwrap_or_default();
        ops.append(&mut node_ops);

        if !ops.is_empty() {
            canvas.history.record(crate::ops::GraphTransaction {
                label: Some("Move Group".to_string()),
                ops,
            });
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(drag) = canvas.interaction.node_drag.take() {
        let geom = canvas.canvas_geometry(&*cx.app, snapshot);
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

                    let rect = node_geom.rect;
                    let rect_min_x = rect.origin.x.0;
                    let rect_min_y = rect.origin.y.0;
                    let rect_max_x = rect.origin.x.0 + rect.size.width.0;
                    let rect_max_y = rect.origin.y.0 + rect.size.height.0;

                    let mut best: Option<(GroupId, f32)> = None;
                    for (group_id, group) in &graph.groups {
                        let gx0 = group.rect.origin.x;
                        let gy0 = group.rect.origin.y;
                        let gx1 = group.rect.origin.x + group.rect.size.width;
                        let gy1 = group.rect.origin.y + group.rect.size.height;
                        if rect_min_x >= gx0
                            && rect_min_y >= gy0
                            && rect_max_x <= gx1
                            && rect_max_y <= gy1
                        {
                            let area = (group.rect.size.width.max(0.0))
                                * (group.rect.size.height.max(0.0));
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

        if !parent_changes.is_empty() {
            let _ = canvas.graph.update(cx.app, |graph, _cx| {
                for (node_id, _from, to) in &parent_changes {
                    if let Some(node) = graph.nodes.get_mut(node_id) {
                        node.parent = *to;
                    }
                }
            });
        }

        let mut ops = canvas
            .graph
            .read_ref(cx.app, |g| {
                drag.nodes
                    .iter()
                    .filter_map(|(id, start)| {
                        let end = g.nodes.get(id).map(|n| n.pos)?;
                        (end != *start).then_some(GraphOp::SetNodePos {
                            id: *id,
                            from: *start,
                            to: end,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .ok()
            .unwrap_or_default();

        for (node_id, from, to) in &parent_changes {
            ops.push(GraphOp::SetNodeParent {
                id: *node_id,
                from: *from,
                to: *to,
            });
        }
        if !ops.is_empty() {
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
            canvas.history.record(crate::ops::GraphTransaction {
                label: Some(label.to_string()),
                ops,
            });
        }
        canvas.interaction.pending_node_drag = None;
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_group_drag.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_group_resize.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(pending) = canvas.interaction.pending_node_drag.take() {
        canvas.interaction.pending_node_resize = None;
        canvas.interaction.snap_guides = None;

        if pending.select_action != PendingNodeSelectAction::None {
            let dx = position.x.0 - pending.start_pos.x.0;
            let dy = position.y.0 - pending.start_pos.y.0;
            let click_distance = snapshot.interaction.node_click_distance.max(0.0);
            let is_click =
                click_distance == 0.0 || (dx * dx + dy * dy) <= click_distance * click_distance;

            if is_click {
                let node = pending.primary;
                canvas.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_groups.clear();

                    match pending.select_action {
                        PendingNodeSelectAction::Toggle => {
                            if let Some(ix) = s.selected_nodes.iter().position(|id| *id == node) {
                                s.selected_nodes.remove(ix);
                            } else {
                                s.selected_nodes.push(node);
                            }
                        }
                        PendingNodeSelectAction::Add => {
                            if !s.selected_nodes.iter().any(|id| *id == node) {
                                s.selected_nodes.push(node);
                            }
                        }
                        PendingNodeSelectAction::None => {}
                    }

                    s.draw_order.retain(|id| *id != node);
                    s.draw_order.push(node);
                });
            }
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_node_resize.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_wire_drag.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom) {
        return true;
    }

    if super::edge_drag::handle_edge_left_up(canvas, cx) {
        return true;
    }

    false
}
