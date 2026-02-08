use std::collections::{BTreeMap, HashMap};

use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, MouseButton, Point, Rect};
use fret_ui::UiHost;

use crate::core::GroupId;
use crate::ops::GraphOp;
use crate::runtime::callbacks::{NodeDragEndOutcome, ViewportMoveEndOutcome, ViewportMoveKind};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingNodeSelectAction, ViewSnapshot, WireDrag, WireDragKind};

pub(super) fn handle_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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
    canvas.interaction.last_canvas_pos = Some(crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    });

    if button == MouseButton::Left
        && canvas.interaction.sticky_wire_ignore_next_up
        && canvas.interaction.wire_drag.is_some()
    {
        canvas.interaction.sticky_wire_ignore_next_up = false;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.panning && canvas.interaction.panning_button == Some(button) {
        canvas.interaction.panning = false;
        canvas.interaction.panning_button = None;
        canvas.interaction.pan_last_screen_pos = None;
        canvas.interaction.pan_last_sample_at = None;
        canvas.stop_auto_pan_timer(cx.app);
        let started_inertia = canvas.maybe_start_pan_inertia_timer(cx.app, cx.window, snapshot);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanDrag,
            ViewportMoveEndOutcome::Ended,
        );
        if started_inertia {
            canvas.emit_move_start(snapshot, ViewportMoveKind::PanInertia);
        }
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

        let end_pos = resize.current_node_pos;
        let end_size = resize.current_size_opt;

        let mut ops: Vec<GraphOp> = Vec::new();
        if resize.start_node_pos != end_pos {
            ops.push(GraphOp::SetNodePos {
                id: resize.node,
                from: resize.start_node_pos,
                to: end_pos,
            });
        }
        if resize.start_size_opt != end_size {
            ops.push(GraphOp::SetNodeSize {
                id: resize.node,
                from: resize.start_size_opt,
                to: end_size,
            });
        }

        let group_rect_ops: Vec<GraphOp> = canvas
            .graph
            .read_ref(cx.app, |graph| {
                resize
                    .current_groups
                    .iter()
                    .filter_map(|(id, to)| {
                        let from = graph.groups.get(id).map(|g| g.rect)?;
                        (from != *to).then_some(GraphOp::SetGroupRect {
                            id: *id,
                            from,
                            to: *to,
                        })
                    })
                    .collect()
            })
            .ok()
            .unwrap_or_default();
        ops.extend(group_rect_ops);

        if !ops.is_empty() {
            let _ = canvas.commit_ops(cx.app, cx.window, Some("Resize Node"), ops);
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(resize) = canvas.interaction.group_resize.take() {
        canvas.interaction.pending_group_resize = None;

        let end = resize.current_rect;
        if end != resize.start_rect {
            let _ = canvas.commit_ops(
                cx.app,
                cx.window,
                Some("Resize Group"),
                vec![GraphOp::SetGroupRect {
                    id: resize.group,
                    from: resize.start_rect,
                    to: end,
                }],
            );
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(drag) = canvas.interaction.group_drag.take() {
        canvas.interaction.pending_group_drag = None;

        let mut ops: Vec<GraphOp> = Vec::new();
        let end_rect = drag.current_rect;
        if end_rect != drag.start_rect {
            ops.push(GraphOp::SetGroupRect {
                id: drag.group,
                from: drag.start_rect,
                to: end_rect,
            });
        }

        for (id, start) in &drag.nodes {
            let end = drag
                .current_nodes
                .iter()
                .find(|(node_id, _)| node_id == id)
                .map(|(_, p)| *p)
                .unwrap_or(*start);
            if end != *start {
                ops.push(GraphOp::SetNodePos {
                    id: *id,
                    from: *start,
                    to: end,
                });
            }
        }

        if !ops.is_empty() {
            let _ = canvas.commit_ops(cx.app, cx.window, Some("Move Group"), ops);
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(drag) = canvas.interaction.node_drag.take() {
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
                        let group_rect =
                            group_overrides.get(group_id).copied().unwrap_or(group.rect);
                        let gx0 = group_rect.origin.x;
                        let gy0 = group_rect.origin.y;
                        let gx1 = group_rect.origin.x + group_rect.size.width;
                        let gy1 = group_rect.origin.y + group_rect.size.height;
                        if rect_min_x >= gx0
                            && rect_min_y >= gy0
                            && rect_max_x <= gx1
                            && rect_max_y <= gy1
                        {
                            let area = (group_rect.size.width.max(0.0))
                                * (group_rect.size.height.max(0.0));
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
            let click_distance_screen = snapshot.interaction.node_click_distance.max(0.0);
            let click_distance_canvas = canvas_units_from_screen_px(click_distance_screen, zoom);
            let is_click = click_distance_screen == 0.0
                || (dx * dx + dy * dy) <= click_distance_canvas * click_distance_canvas;

            if is_click {
                let node = pending.primary;
                canvas.update_view_state(cx.app, |s| {
                    match pending.select_action {
                        PendingNodeSelectAction::Toggle => {
                            if let Some(ix) = s.selected_nodes.iter().position(|id| *id == node) {
                                s.selected_nodes.remove(ix);
                            } else {
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

    if let Some(pending) = canvas.interaction.pending_wire_drag.take() {
        if snapshot.interaction.connect_on_click {
            if matches!(pending.kind, WireDragKind::New { .. }) {
                let kind = pending.kind.clone();
                canvas.interaction.wire_drag = Some(WireDrag {
                    kind: pending.kind,
                    pos: position,
                });
                canvas.interaction.click_connect = true;
                canvas.emit_connect_start(snapshot, &kind);
            }
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom) {
        return true;
    }

    if super::edge_insert_drag::handle_edge_insert_left_up(canvas, cx, position) {
        return true;
    }

    if super::edge_drag::handle_edge_left_up(canvas, cx) {
        return true;
    }

    false
}
