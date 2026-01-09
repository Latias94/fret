use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::{EdgeId, GroupId, NodeId as GraphNodeId, PortId};

use super::super::state::{
    EdgeDrag, PendingGroupDrag, PendingGroupResize, PendingNodeDrag, PendingNodeResize,
    PendingWireDrag, ViewSnapshot, WireDragKind,
};
use super::NodeGraphCanvas;

pub(super) fn handle_left_click_pointer_down<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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
        Resize(GraphNodeId, Rect),
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
                let mut scratch_ports: Vec<PortId> = Vec::new();
                let mut scratch_edges: Vec<EdgeId> = Vec::new();
                if let Some(port) = this.hit_port(
                    geom.as_ref(),
                    index.as_ref(),
                    position,
                    zoom,
                    &mut scratch_ports,
                ) {
                    return Hit::Port(port);
                }
                let order = geom.order.clone();
                let Some(node) = order.iter().rev().find_map(|id| {
                    geom.nodes
                        .get(id)
                        .is_some_and(|ng| ng.rect.contains(position))
                        .then_some(*id)
                }) else {
                    if let Some(edge) = this.hit_edge(
                        graph,
                        snapshot,
                        geom.as_ref(),
                        index.as_ref(),
                        position,
                        zoom,
                        &mut scratch_edges,
                    ) {
                        return Hit::Edge(edge);
                    }

                    let order =
                        super::super::geometry::group_order(graph, &snapshot.group_draw_order);
                    for group_id in order.iter().rev() {
                        let Some(group) = graph.groups.get(group_id) else {
                            continue;
                        };
                        let rect = super::group_resize::group_rect_to_px(group.rect);
                        let handle = this.resize_handle_rect(rect, zoom);
                        if super::group_resize::group_resize_handle_hit(handle, position, zoom, 6.0)
                        {
                            return Hit::GroupResize(*group_id, group.rect);
                        }
                    }

                    let header_h = this.style.node_header_height;
                    for group_id in order.iter().rev() {
                        let Some(group) = graph.groups.get(group_id) else {
                            continue;
                        };
                        if !super::pending_group_drag::group_header_hit(
                            group.rect, header_h, zoom, position,
                        ) {
                            continue;
                        }
                        return Hit::GroupHeader(*group_id, group.rect);
                    }
                    return Hit::Background;
                };
                let Some(rect) = geom.nodes.get(&node).map(|ng| ng.rect) else {
                    return Hit::Background;
                };
                let handle = this.resize_handle_rect(rect, zoom);
                let is_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
                if is_selected && NodeGraphCanvas::rect_contains(handle, position) {
                    Hit::Resize(node, rect)
                } else {
                    Hit::Node(node, rect)
                }
            })
            .unwrap_or(Hit::Background)
    };

    match hit {
        Hit::Port(port) => {
            canvas.interaction.pending_group_drag = None;
            canvas.interaction.group_drag = None;
            canvas.interaction.pending_group_resize = None;
            canvas.interaction.group_resize = None;
            canvas.interaction.pending_node_drag = None;
            canvas.interaction.node_drag = None;
            canvas.interaction.pending_wire_drag = None;
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            let yank = (modifiers.ctrl || modifiers.meta).then(|| {
                let this = &*canvas;
                this.graph
                    .read_ref(cx.app, |graph| {
                        NodeGraphCanvas::yank_edges_from_port(graph, port)
                    })
                    .ok()
                    .unwrap_or_default()
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

            canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
                kind,
                start_pos: position,
            });
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        Hit::Resize(node, rect) => {
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
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;

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

            let start_size = crate::core::CanvasSize {
                width: rect.size.width.0 * zoom,
                height: rect.size.height.0 * zoom,
            };
            let start_size_opt = canvas
                .graph
                .read_ref(cx.app, |g| g.nodes.get(&node).and_then(|n| n.size))
                .ok()
                .flatten();

            canvas.interaction.pending_node_resize = Some(PendingNodeResize {
                node,
                start_pos: position,
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
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            let offset = Point::new(
                Px(position.x.0 - rect.origin.x.0),
                Px(position.y.0 - rect.origin.y.0),
            );
            let already_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
            let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.meta;

            canvas.update_view_state(cx.app, |s| {
                s.selected_edges.clear();
                s.selected_groups.clear();
                if multi_mod {
                    if let Some(ix) = s.selected_nodes.iter().position(|id| *id == node) {
                        s.selected_nodes.remove(ix);
                    } else {
                        s.selected_nodes.push(node);
                    }
                } else if !s.selected_nodes.iter().any(|id| *id == node) {
                    s.selected_nodes.clear();
                    s.selected_nodes.push(node);
                }
                s.draw_order.retain(|id| *id != node);
                s.draw_order.push(node);
            });

            if !multi_mod {
                let nodes_for_drag = (already_selected && snapshot.selected_nodes.len() > 1)
                    .then(|| snapshot.selected_nodes.clone())
                    .unwrap_or_else(|| vec![node]);
                canvas.interaction.pending_node_drag = Some(PendingNodeDrag {
                    primary: node,
                    nodes: nodes_for_drag,
                    grab_offset: offset,
                    start_pos: position,
                });
                cx.capture_pointer(cx.node);
            }

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
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            let multi = modifiers.ctrl || modifiers.meta;
            canvas.update_view_state(cx.app, |s| {
                s.selected_nodes.clear();
                s.selected_groups.clear();
                if multi {
                    if let Some(ix) = s.selected_edges.iter().position(|id| *id == edge) {
                        s.selected_edges.remove(ix);
                    } else {
                        s.selected_edges.push(edge);
                    }
                } else {
                    s.selected_edges.clear();
                    s.selected_edges.push(edge);
                }
            });
            canvas.interaction.edge_drag = Some(EdgeDrag {
                edge,
                start_pos: position,
            });
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
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;

            let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.meta;
            canvas.update_view_state(cx.app, |s| {
                s.selected_nodes.clear();
                s.selected_edges.clear();
                if multi_mod {
                    if let Some(ix) = s.selected_groups.iter().position(|id| *id == group) {
                        s.selected_groups.remove(ix);
                    } else {
                        s.selected_groups.push(group);
                    }
                } else if !s.selected_groups.iter().any(|id| *id == group) {
                    s.selected_groups.clear();
                    s.selected_groups.push(group);
                }
                s.group_draw_order.retain(|id| *id != group);
                s.group_draw_order.push(group);
            });

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
            canvas.interaction.edge_drag = None;
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;

            let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.meta;
            canvas.update_view_state(cx.app, |s| {
                s.selected_nodes.clear();
                s.selected_edges.clear();
                if multi_mod {
                    if let Some(ix) = s.selected_groups.iter().position(|id| *id == group) {
                        s.selected_groups.remove(ix);
                    } else {
                        s.selected_groups.push(group);
                    }
                } else if !s.selected_groups.iter().any(|id| *id == group) {
                    s.selected_groups.clear();
                    s.selected_groups.push(group);
                }
                s.group_draw_order.retain(|id| *id != group);
                s.group_draw_order.push(group);
            });

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
            canvas.interaction.pending_marquee = None;
            canvas.interaction.marquee = None;
            canvas.interaction.hover_port = None;
            canvas.interaction.hover_port_valid = false;
            canvas.interaction.hover_port_convertible = false;
            super::marquee::begin_background_marquee(canvas, cx, snapshot, position, modifiers);
        }
    }

    true
}
