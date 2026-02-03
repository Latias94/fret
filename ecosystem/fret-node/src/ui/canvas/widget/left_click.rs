use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::{EdgeId, GroupId, NodeId as GraphNodeId, PortId};
use crate::interaction::NodeGraphDragHandleMode;
use crate::rules::EdgeEndpoint;

use super::super::state::{
    EdgeDrag, NodeResizeHandle, PendingEdgeInsertDrag, PendingGroupDrag, PendingGroupResize,
    PendingNodeDrag, PendingNodeResize, PendingNodeSelectAction, PendingWireDrag, ViewSnapshot,
    WireDragKind,
};
use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

fn node_header_hit(rect: Rect, header_height_screen: f32, zoom: f32, position: Point) -> bool {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let header_h = (header_height_screen / zoom).max(0.0);
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = rect.origin.x.0 + rect.size.width.0;
    let y1 = rect.origin.y.0 + header_h.min(rect.size.height.0.max(0.0));

    position.x.0 >= x0
        && position.y.0 >= y0
        && position.x.0 <= x1
        && position.y.0 <= y1
        && header_h > 0.0
}

pub(super) fn handle_left_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    canvas.interaction.hover_edge = None;

    #[derive(Debug, Clone, Copy)]
    enum Hit {
        Port(PortId),
        EdgeAnchor(EdgeId, EdgeEndpoint, PortId),
        Resize(GraphNodeId, Rect, NodeResizeHandle),
        Node(GraphNodeId, Rect),
        Edge(EdgeId),
        GroupResize(GroupId, crate::core::CanvasRect),
        GroupHeader(GroupId, crate::core::CanvasRect),
        Background,
    }

    let hit = {
        let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
        let this = &*canvas;
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch = HitTestScratch::default();
                let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                if let Some(port) = this.hit_port(&mut ctx, position) {
                    return Hit::Port(port);
                }

                if let Some((edge, endpoint, fixed)) =
                    this.hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
                {
                    return Hit::EdgeAnchor(edge, endpoint, fixed);
                }

                let order = geom.order.clone();
                let Some(node) = order.iter().rev().find_map(|id| {
                    geom.nodes
                        .get(id)
                        .is_some_and(|ng| ng.rect.contains(position))
                        .then_some(*id)
                }) else {
                    if let Some(edge) = this.hit_edge(graph, snapshot, &mut ctx, position) {
                        return Hit::Edge(edge);
                    }

                    let order =
                        super::super::geometry::group_order(graph, &snapshot.group_draw_order);
                    for group_id in order.iter().rev() {
                        let Some(group) = graph.groups.get(group_id) else {
                            continue;
                        };
                        let rect0 = this.group_rect_with_preview(*group_id, group.rect);
                        let rect = super::group_resize::group_rect_to_px(rect0);
                        let handle = this.resize_handle_rect(rect, zoom);
                        if super::group_resize::group_resize_handle_hit(handle, position, zoom, 6.0)
                        {
                            return Hit::GroupResize(*group_id, rect0);
                        }
                    }

                    let header_h = this.style.node_header_height;
                    for group_id in order.iter().rev() {
                        let Some(group) = graph.groups.get(group_id) else {
                            continue;
                        };
                        let rect0 = this.group_rect_with_preview(*group_id, group.rect);
                        if !super::pending_group_drag::group_header_hit(
                            rect0, header_h, zoom, position,
                        ) {
                            continue;
                        }
                        return Hit::GroupHeader(*group_id, rect0);
                    }
                    return Hit::Background;
                };
                let Some(rect) = geom.nodes.get(&node).map(|ng| ng.rect) else {
                    return Hit::Background;
                };
                let is_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
                if is_selected {
                    let resize_handles =
                        this.presenter.node_resize_handles(graph, node, &this.style);
                    for handle in NodeResizeHandle::ALL {
                        if !resize_handles.contains(handle) {
                            continue;
                        }
                        let hit_rect = this.node_resize_handle_rect(rect, handle, zoom);
                        if NodeGraphCanvasWith::<M>::rect_contains(hit_rect, position) {
                            return Hit::Resize(node, rect, handle);
                        }
                    }
                }

                Hit::Node(node, rect)
            })
            .unwrap_or(Hit::Background)
    };

    let selection_key_pressed = snapshot.interaction.selection_key.is_pressed(modifiers);
    canvas.interaction.multi_selection_active = snapshot
        .interaction
        .multi_selection_key
        .is_pressed(modifiers);
    let multi_selection_pressed = canvas.interaction.multi_selection_active;

    if snapshot.interaction.elements_selectable
        && selection_key_pressed
        && !matches!(
            hit,
            Hit::Port(_) | Hit::EdgeAnchor(_, _, _) | Hit::Resize(_, _, _) | Hit::GroupResize(_, _)
        )
        && !matches!(hit, Hit::Background)
    {
        canvas.interaction.edge_drag = None;
        canvas.interaction.pending_edge_insert_drag = None;
        canvas.interaction.edge_insert_drag = None;
        canvas.interaction.pending_group_drag = None;
        canvas.interaction.group_drag = None;
        canvas.interaction.pending_group_resize = None;
        canvas.interaction.group_resize = None;
        canvas.interaction.pending_node_drag = None;
        canvas.interaction.node_drag = None;
        canvas.interaction.pending_node_resize = None;
        canvas.interaction.node_resize = None;
        canvas.interaction.pending_wire_drag = None;
        canvas.interaction.wire_drag = None;
        canvas.interaction.click_connect = false;
        canvas.interaction.pending_marquee = None;
        canvas.interaction.marquee = None;
        canvas.interaction.focused_edge = None;
        canvas.interaction.hover_port = None;
        canvas.interaction.hover_port_valid = false;
        canvas.interaction.hover_port_convertible = false;
        canvas.interaction.hover_port_diagnostic = None;

        super::marquee::begin_background_marquee(canvas, cx, snapshot, position, modifiers, false);
        return true;
    }

    match hit {
        Hit::Port(port) => {
            canvas.interaction.focused_edge = None;
            let port_base_connectable = canvas
                .graph
                .read_ref(cx.app, |graph| {
                    NodeGraphCanvasWith::<M>::port_is_connectable_base(
                        graph,
                        &snapshot.interaction,
                        port,
                    )
                })
                .ok()
                .unwrap_or(false);
            let port_connectable_start = port_base_connectable
                && canvas
                    .graph
                    .read_ref(cx.app, |graph| {
                        NodeGraphCanvasWith::<M>::port_is_connectable_start(
                            graph,
                            &snapshot.interaction,
                            port,
                        )
                    })
                    .ok()
                    .unwrap_or(false);
            let port_connectable_end = port_base_connectable
                && canvas
                    .graph
                    .read_ref(cx.app, |graph| {
                        NodeGraphCanvasWith::<M>::port_is_connectable_end(
                            graph,
                            &snapshot.interaction,
                            port,
                        )
                    })
                    .ok()
                    .unwrap_or(false);

            if snapshot.interaction.connect_on_click
                && canvas.interaction.click_connect
                && canvas.interaction.wire_drag.is_some()
            {
                // When click-to-connect is active, ignore clicks on non-connectable ports so we do
                // not accidentally trigger the "drop on empty" picker.
                if !port_connectable_end {
                    return true;
                }
                if let Some(mut w) = canvas.interaction.wire_drag.take() {
                    w.pos = position;
                    canvas.interaction.wire_drag = Some(w);
                    canvas.interaction.click_connect = false;
                    canvas.interaction.pending_wire_drag = None;
                    let _ = super::wire_drag::handle_wire_left_up_with_forced_target(
                        canvas,
                        cx,
                        snapshot,
                        zoom,
                        Some(port),
                    );
                    return true;
                }
            }

            if !port_base_connectable {
                canvas.interaction.click_connect = false;
                return true;
            }

            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.focused_edge = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;
            let yank = (modifiers.ctrl || modifiers.meta).then(|| {
                canvas.yank_reconnectable_edges_from_port(cx.app, &snapshot.interaction, port)
            });

            let kind = match yank {
                Some(edges) if edges.len() > 1 => WireDragKind::ReconnectMany { edges },
                Some(mut edges) if edges.len() == 1 => {
                    let (edge, endpoint, fixed) = edges.remove(0);
                    WireDragKind::Reconnect {
                        edge,
                        endpoint,
                        fixed,
                    }
                }
                _ => WireDragKind::New {
                    from: port,
                    bundle: vec![port],
                },
            };

            if matches!(kind, WireDragKind::New { .. }) && !port_connectable_start {
                return true;
            }

            canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
                kind,
                start_pos: position,
            });
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::EdgeAnchor(edge, endpoint, fixed) => {
            let edge_selectable = canvas
                .graph
                .read_ref(cx.app, |g| {
                    NodeGraphCanvasWith::<M>::edge_is_selectable(g, &snapshot.interaction, edge)
                })
                .ok()
                .unwrap_or(false);

            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.focused_edge =
                (snapshot.interaction.edges_focusable && edge_selectable).then_some(edge);
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_edge = None;

            if edge_selectable {
                let multi = multi_selection_pressed;
                canvas.update_view_state(cx.app, |s| {
                    if multi {
                        if let Some(ix) = s.selected_edges.iter().position(|id| *id == edge) {
                            s.selected_edges.remove(ix);
                        } else {
                            s.selected_edges.push(edge);
                        }
                    } else {
                        s.selected_nodes.clear();
                        s.selected_groups.clear();
                        s.selected_edges.clear();
                        s.selected_edges.push(edge);
                    }
                });
            }

            canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
                kind: WireDragKind::Reconnect {
                    edge,
                    endpoint,
                    fixed,
                },
                start_pos: position,
            });
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::Resize(node, rect, handle) => {
            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;

            if snapshot.interaction.elements_selectable {
                canvas.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_groups.clear();
                    if !s.selected_nodes.iter().any(|id| *id == node) {
                        s.selected_nodes.clear();
                        s.selected_nodes.push(node);
                    }
                    s.draw_order.retain(|id| *id != node);
                    s.draw_order.push(node);
                });
            }

            let start_size = crate::core::CanvasSize {
                width: rect.size.width.0 * zoom,
                height: rect.size.height.0 * zoom,
            };
            let start_size_opt = canvas
                .graph
                .read_ref(cx.app, |g| g.nodes.get(&node).and_then(|n| n.size))
                .ok()
                .flatten();
            let start_node_pos = canvas
                .graph
                .read_ref(cx.app, |g| g.nodes.get(&node).map(|n| n.pos))
                .ok()
                .flatten()
                .unwrap_or_default();

            canvas.interaction.pending_node_resize = Some(PendingNodeResize {
                node,
                handle,
                start_pos: position,
                start_node_pos,
                start_size,
                start_size_opt,
            });
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::Node(node, rect) => {
            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.focused_edge = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;
            let offset = Point::new(
                Px(position.x.0 - rect.origin.x.0),
                Px(position.y.0 - rect.origin.y.0),
            );
            let already_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
            let node_selectable = canvas
                .graph
                .read_ref(cx.app, |g| {
                    NodeGraphCanvasWith::<M>::node_is_selectable(g, &snapshot.interaction, node)
                })
                .ok()
                .unwrap_or(false);
            let node_draggable = canvas
                .graph
                .read_ref(cx.app, |g| {
                    NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, node)
                })
                .ok()
                .unwrap_or(false);
            let select_action = if node_selectable && multi_selection_pressed {
                PendingNodeSelectAction::Toggle
            } else {
                PendingNodeSelectAction::None
            };

            if node_selectable && !multi_selection_pressed {
                canvas.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_groups.clear();
                    if !s.selected_nodes.iter().any(|id| *id == node) {
                        s.selected_nodes.clear();
                        s.selected_nodes.push(node);
                    }
                    s.draw_order.retain(|id| *id != node);
                    s.draw_order.push(node);
                });
            }

            let nodes_for_drag = if node_draggable
                && node_selectable
                && already_selected
                && snapshot.selected_nodes.len() > 1
            {
                snapshot.selected_nodes.clone()
            } else {
                vec![node]
            };
            let nodes_for_drag = canvas
                .graph
                .read_ref(cx.app, |g| {
                    nodes_for_drag
                        .iter()
                        .copied()
                        .filter(|id| {
                            NodeGraphCanvasWith::<M>::node_is_draggable(
                                g,
                                &snapshot.interaction,
                                *id,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .ok()
                .unwrap_or_else(|| vec![node]);
            let drag_enabled = match snapshot.interaction.node_drag_handle_mode {
                NodeGraphDragHandleMode::Any => true,
                NodeGraphDragHandleMode::Header => {
                    node_header_hit(rect, canvas.style.node_header_height, zoom, position)
                }
            };
            let drag_enabled = drag_enabled && node_draggable;
            canvas.interaction.pending_node_drag = Some(PendingNodeDrag {
                primary: node,
                nodes: nodes_for_drag,
                grab_offset: offset,
                start_pos: position,
                select_action,
                drag_enabled,
            });
            cx.capture_pointer(cx.node);

            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::Edge(edge) => {
            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;
            let multi = multi_selection_pressed;
            let edge_selectable = canvas
                .graph
                .read_ref(cx.app, |g| {
                    NodeGraphCanvasWith::<M>::edge_is_selectable(g, &snapshot.interaction, edge)
                })
                .ok()
                .unwrap_or(false);
            if edge_selectable {
                canvas.update_view_state(cx.app, |s| {
                    if multi {
                        if let Some(ix) = s.selected_edges.iter().position(|id| *id == edge) {
                            s.selected_edges.remove(ix);
                        } else {
                            s.selected_edges.push(edge);
                        }
                    } else {
                        s.selected_nodes.clear();
                        s.selected_groups.clear();
                        s.selected_edges.clear();
                        s.selected_edges.push(edge);
                    }
                });
            }
            canvas.interaction.focused_edge =
                (snapshot.interaction.edges_focusable && edge_selectable).then_some(edge);

            if snapshot.interaction.edge_insert_on_alt_drag && (modifiers.alt || modifiers.alt_gr) {
                canvas.interaction.pending_edge_insert_drag = Some(PendingEdgeInsertDrag {
                    edge,
                    start_pos: position,
                });
                canvas.interaction.edge_insert_drag = None;
                canvas.interaction.edge_drag = None;
            } else {
                canvas.interaction.pending_edge_insert_drag = None;
                canvas.interaction.edge_insert_drag = None;
                canvas.interaction.edge_drag = Some(EdgeDrag {
                    edge,
                    start_pos: position,
                });
            }
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::GroupResize(group, rect) => {
            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.focused_edge = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;

            if snapshot.interaction.elements_selectable {
                canvas.update_view_state(cx.app, |s| {
                    if multi_selection_pressed {
                        if let Some(ix) = s.selected_groups.iter().position(|id| *id == group) {
                            s.selected_groups.remove(ix);
                        } else {
                            s.selected_groups.push(group);
                        }
                    } else if !s.selected_groups.iter().any(|id| *id == group) {
                        s.selected_nodes.clear();
                        s.selected_edges.clear();
                        s.selected_groups.clear();
                        s.selected_groups.push(group);
                    }
                    s.group_draw_order.retain(|id| *id != group);
                    s.group_draw_order.push(group);
                });
            }

            canvas.interaction.pending_group_resize = Some(PendingGroupResize {
                group,
                start_pos: position,
                start_rect: rect,
            });
            canvas.interaction.group_resize = None;

            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::GroupHeader(group, rect) => {
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.focused_edge = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;

            if snapshot.interaction.elements_selectable {
                canvas.update_view_state(cx.app, |s| {
                    if multi_selection_pressed {
                        if let Some(ix) = s.selected_groups.iter().position(|id| *id == group) {
                            s.selected_groups.remove(ix);
                        } else {
                            s.selected_groups.push(group);
                        }
                    } else if !s.selected_groups.iter().any(|id| *id == group) {
                        s.selected_nodes.clear();
                        s.selected_edges.clear();
                        s.selected_groups.clear();
                        s.selected_groups.push(group);
                    }
                    s.group_draw_order.retain(|id| *id != group);
                    s.group_draw_order.push(group);
                });
            }

            canvas.interaction.pending_group_drag = Some(PendingGroupDrag {
                group,
                start_pos: position,
                start_rect: rect,
            });
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;

            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::Background => {
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_edge_insert_drag = None;
            canvas.interaction.edge_insert_drag = None;
            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_node_resize = None;
            canvas.interaction.node_resize = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.wire_drag = None;
            canvas.interaction.click_connect = false;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.focused_edge = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            canvas.interaction.hover_port_diagnostic = None;
            if snapshot.interaction.elements_selectable {
                // XyFlow semantics: background drags pan by default, and selection boxes are
                // activated by Shift unless `selection_on_drag` is enabled.
                //
                // We still begin a pending marquee so a click (no drag) can clear selection. Once
                // the drag exceeds the threshold, `marquee::handle_marquee_move` will decide
                // whether to start a selection box or switch into panning.
                super::marquee::begin_background_marquee(
                    canvas, cx, snapshot, position, modifiers, true,
                );
            } else if snapshot.interaction.pan_on_drag.left {
                let _ = super::pan_zoom::begin_panning(
                    canvas,
                    cx,
                    snapshot,
                    position,
                    fret_core::MouseButton::Left,
                );
            }
        }
    }

    true
}
