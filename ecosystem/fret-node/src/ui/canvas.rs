use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, PathCommand, PathConstraints, PathStyle,
    Point, Px, Rect, SceneOp, Size, StrokeStyle, TextBlobId, TextConstraints, TextOverflow,
    TextWrap, Transform2D,
};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};
use serde_json::Value;

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId as GraphNodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::io::NodeGraphViewState;
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction, apply_transaction};
use crate::rules::EdgeEndpoint;

use super::presenter::{
    DefaultNodeGraphPresenter, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
    NodeGraphPresenter,
};
use super::style::NodeGraphStyle;

#[derive(Debug, Clone)]
struct ViewSnapshot {
    pan: CanvasPoint,
    zoom: f32,
    selected_nodes: Vec<GraphNodeId>,
    selected_edges: Vec<EdgeId>,
    draw_order: Vec<GraphNodeId>,
}

#[derive(Debug, Default, Clone)]
struct InteractionState {
    last_pos: Option<Point>,
    panning: bool,
    node_drag: Option<NodeDrag>,
    wire_drag: Option<WireDrag>,
    edge_drag: Option<EdgeDrag>,
    hover_edge: Option<EdgeId>,
    context_menu: Option<ContextMenuState>,
}

#[derive(Debug, Clone)]
struct NodeDrag {
    node: GraphNodeId,
    grab_offset: Point,
}

#[derive(Debug, Clone)]
enum WireDragKind {
    New {
        from: PortId,
    },
    Reconnect {
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        fixed: PortId,
    },
}

#[derive(Debug, Clone)]
struct WireDrag {
    kind: WireDragKind,
    pos: Point,
}

#[derive(Debug, Clone)]
struct EdgeDrag {
    edge: EdgeId,
    start_pos: Point,
}

#[derive(Debug, Clone)]
enum ContextMenuTarget {
    Edge(EdgeId),
}

#[derive(Debug, Clone)]
struct ContextMenuState {
    origin: Point,
    invoked_at: Point,
    target: ContextMenuTarget,
    items: Vec<NodeGraphContextMenuItem>,
    hovered_item: Option<usize>,
}

/// Retained node-graph canvas widget (MVP).
///
/// This draws nodes and wires and supports:
/// - pan (MMB drag, wheel without Ctrl),
/// - zoom (Ctrl+wheel, centered),
/// - node drag (LMB),
/// - connect ports (LMB drag pin -> pin).
pub struct NodeGraphCanvas {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    presenter: Box<dyn NodeGraphPresenter>,
    style: NodeGraphStyle,

    cached_pan: CanvasPoint,
    cached_zoom: f32,

    wire_paths: Vec<fret_core::PathId>,
    text_blobs: Vec<TextBlobId>,
    interaction: InteractionState,
}

impl NodeGraphCanvas {
    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        Self {
            graph,
            view_state,
            presenter: Box::new(DefaultNodeGraphPresenter::default()),
            style: NodeGraphStyle::default(),
            cached_pan: CanvasPoint::default(),
            cached_zoom: 1.0,
            wire_paths: Vec::new(),
            text_blobs: Vec::new(),
            interaction: InteractionState::default(),
        }
    }

    pub fn with_presenter(mut self, presenter: impl NodeGraphPresenter + 'static) -> Self {
        self.presenter = Box::new(presenter);
        self
    }

    pub fn with_style(mut self, style: NodeGraphStyle) -> Self {
        self.style = style;
        self
    }

    fn sync_view_state<H: UiHost>(&mut self, host: &mut H) -> ViewSnapshot {
        let mut snapshot = ViewSnapshot {
            pan: self.cached_pan,
            zoom: self.cached_zoom,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            draw_order: Vec::new(),
        };

        let _ = self.view_state.read(host, |_host, s| {
            snapshot.pan = s.pan;
            snapshot.zoom = s.zoom;
            snapshot.selected_nodes = s.selected_nodes.clone();
            snapshot.selected_edges = s.selected_edges.clone();
            snapshot.draw_order = s.draw_order.clone();
        });

        let zoom = snapshot.zoom;
        if zoom.is_finite() && zoom > 0.0 {
            self.cached_zoom = zoom.clamp(self.style.min_zoom, self.style.max_zoom);
        } else {
            self.cached_zoom = 1.0;
        }
        self.cached_pan = snapshot.pan;
        snapshot.zoom = self.cached_zoom;
        snapshot.pan = self.cached_pan;

        snapshot
    }

    fn update_view_state<H: UiHost>(
        &mut self,
        host: &mut H,
        f: impl FnOnce(&mut NodeGraphViewState),
    ) {
        let _ = self.view_state.update(host, |s, _cx| {
            f(s);
        });
        self.sync_view_state(host);
    }

    fn apply_ops<H: UiHost>(&mut self, host: &mut H, ops: Vec<GraphOp>) {
        if ops.is_empty() {
            return;
        }
        let tx = GraphTransaction { label: None, ops };
        let _ = self
            .graph
            .update(host, |g, _cx| match apply_transaction(g, &tx) {
                Ok(()) => {}
                Err(err) => {
                    tracing::warn!("failed to apply node-graph ops: {err}");
                }
            });
    }

    fn node_order(&self, graph: &Graph, snapshot: &ViewSnapshot) -> Vec<GraphNodeId> {
        let mut seen: HashSet<GraphNodeId> = HashSet::new();
        let mut out: Vec<GraphNodeId> = Vec::new();

        for id in &snapshot.draw_order {
            if graph.nodes.contains_key(id) && seen.insert(*id) {
                out.push(*id);
            }
        }

        for id in graph.nodes.keys() {
            if seen.insert(*id) {
                out.push(*id);
            }
        }

        out
    }

    fn build_port_centers(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        zoom: f32,
    ) -> HashMap<PortId, Point> {
        let mut out: HashMap<PortId, Point> = HashMap::new();
        let node_order = self.node_order(graph, snapshot);
        for node in &node_order {
            let (inputs, outputs) = self.node_ports(graph, *node);
            for port in inputs.iter().chain(outputs.iter()) {
                if let Some(center) = self.port_center(graph, *node, *port, zoom) {
                    out.insert(*port, center);
                }
            }
        }
        out
    }

    fn node_ports<'a>(&self, graph: &'a Graph, node: GraphNodeId) -> (Vec<PortId>, Vec<PortId>) {
        let Some(n) = graph.nodes.get(&node) else {
            return (Vec::new(), Vec::new());
        };
        let mut inputs: Vec<PortId> = Vec::new();
        let mut outputs: Vec<PortId> = Vec::new();
        for port_id in &n.ports {
            let Some(p) = graph.ports.get(port_id) else {
                continue;
            };
            match p.dir {
                PortDirection::In => inputs.push(*port_id),
                PortDirection::Out => outputs.push(*port_id),
            }
        }
        (inputs, outputs)
    }

    fn node_height(&self, inputs: usize, outputs: usize, zoom: f32) -> f32 {
        let rows = inputs.max(outputs) as f32;
        let base = self.style.node_header_height + 2.0 * self.style.node_padding;
        let pin_area = rows * self.style.pin_row_height;
        (base + pin_area) / zoom
    }

    fn node_rect(&self, graph: &Graph, node: GraphNodeId, zoom: f32) -> Option<Rect> {
        let n = graph.nodes.get(&node)?;
        let (inputs, outputs) = self.node_ports(graph, node);
        let w = self.style.node_width / zoom;
        let h = self.node_height(inputs.len(), outputs.len(), zoom);
        Some(Rect::new(
            Point::new(Px(n.pos.x), Px(n.pos.y)),
            Size::new(Px(w), Px(h)),
        ))
    }

    fn port_center(
        &self,
        graph: &Graph,
        node: GraphNodeId,
        port: PortId,
        zoom: f32,
    ) -> Option<Point> {
        let rect = self.node_rect(graph, node, zoom)?;
        let (inputs, outputs) = self.node_ports(graph, node);
        let Some(p) = graph.ports.get(&port) else {
            return None;
        };

        let row = match p.dir {
            PortDirection::In => inputs.iter().position(|id| *id == port)?,
            PortDirection::Out => outputs.iter().position(|id| *id == port)?,
        } as f32;

        let x = match p.dir {
            PortDirection::In => rect.origin.x.0,
            PortDirection::Out => rect.origin.x.0 + rect.size.width.0,
        };

        let y = rect.origin.y.0
            + (self.style.node_header_height + self.style.node_padding) / zoom
            + (row + 0.5) * (self.style.pin_row_height / zoom);

        Some(Point::new(Px(x), Px(y)))
    }

    fn hit_port(&self, graph: &Graph, pos: Point, zoom: f32) -> Option<PortId> {
        let r = self.style.pin_radius / zoom;
        let r2 = r * r;
        for (&port_id, port) in &graph.ports {
            let Some(center) = self.port_center(graph, port.node, port_id, zoom) else {
                continue;
            };
            let dx = center.x.0 - pos.x.0;
            let dy = center.y.0 - pos.y.0;
            if dx * dx + dy * dy <= r2 {
                return Some(port_id);
            }
        }
        None
    }

    fn hit_edge(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        pos: Point,
        zoom: f32,
    ) -> Option<EdgeId> {
        let hit_w = (self.style.wire_interaction_width / zoom).max(self.style.wire_width / zoom);
        let threshold2 = hit_w * hit_w;

        let port_centers = self.build_port_centers(graph, snapshot, zoom);

        let mut best: Option<(EdgeId, f32)> = None;
        for (&edge_id, edge) in &graph.edges {
            let Some(from) = port_centers.get(&edge.from).copied() else {
                continue;
            };
            let Some(to) = port_centers.get(&edge.to).copied() else {
                continue;
            };

            let d2 = wire_distance2(pos, from, to, zoom);
            if d2 <= threshold2 {
                match best {
                    Some((_id, best_d2)) if best_d2 <= d2 => {}
                    _ => best = Some((edge_id, d2)),
                }
            }
        }

        best.map(|(id, _)| id)
    }

    fn clamp_context_menu_origin(
        &self,
        desired: Point,
        item_count: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = context_menu_rect_at(&self.style, desired, item_count, snapshot.zoom);

        let viewport_w = bounds.size.width.0 / snapshot.zoom;
        let viewport_h = bounds.size.height.0 / snapshot.zoom;
        let viewport_origin_x = -snapshot.pan.x;
        let viewport_origin_y = -snapshot.pan.y;

        let min_x = viewport_origin_x;
        let min_y = viewport_origin_y;
        let max_x = viewport_origin_x + (viewport_w - rect.size.width.0).max(0.0);
        let max_y = viewport_origin_y + (viewport_h - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }

    fn activate_context_menu_item<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
    ) {
        match (target, item.action) {
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::InsertReroute) => {
                let planned = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            let edge = graph.edges.get(edge_id)?.clone();
                            let from_port = graph.ports.get(&edge.from)?;
                            let to_port = graph.ports.get(&edge.to)?;

                            let port_kind = match edge.kind {
                                EdgeKind::Data => PortKind::Data,
                                EdgeKind::Exec => PortKind::Exec,
                            };
                            let ty = from_port.ty.clone().or_else(|| to_port.ty.clone());

                            let node_id = GraphNodeId::new();
                            let in_port_id = PortId::new();
                            let out_port_id = PortId::new();
                            let new_edge_id = EdgeId::new();

                            let node = Node {
                                kind: NodeKindKey::new("fret.reroute"),
                                kind_version: 1,
                                pos: CanvasPoint {
                                    x: invoked_at.x.0,
                                    y: invoked_at.y.0,
                                },
                                collapsed: false,
                                ports: Vec::new(),
                                data: Value::default(),
                            };

                            let in_port = Port {
                                node: node_id,
                                key: PortKey::new("in"),
                                dir: PortDirection::In,
                                kind: port_kind,
                                capacity: PortCapacity::Single,
                                ty: ty.clone(),
                                data: Value::default(),
                            };

                            let out_port = Port {
                                node: node_id,
                                key: PortKey::new("out"),
                                dir: PortDirection::Out,
                                kind: port_kind,
                                capacity: PortCapacity::Multi,
                                ty,
                                data: Value::default(),
                            };

                            let old_endpoints = EdgeEndpoints {
                                from: edge.from,
                                to: edge.to,
                            };

                            let mut ops: Vec<GraphOp> = Vec::new();
                            ops.push(GraphOp::AddNode { id: node_id, node });
                            ops.push(GraphOp::AddPort {
                                id: in_port_id,
                                port: in_port,
                            });
                            ops.push(GraphOp::AddPort {
                                id: out_port_id,
                                port: out_port,
                            });
                            ops.push(GraphOp::SetNodePorts {
                                id: node_id,
                                from: Vec::new(),
                                to: vec![in_port_id, out_port_id],
                            });

                            ops.push(GraphOp::SetEdgeEndpoints {
                                id: *edge_id,
                                from: old_endpoints,
                                to: EdgeEndpoints {
                                    from: edge.from,
                                    to: in_port_id,
                                },
                            });

                            ops.push(GraphOp::AddEdge {
                                id: new_edge_id,
                                edge: Edge {
                                    kind: edge.kind,
                                    from: out_port_id,
                                    to: edge.to,
                                },
                            });

                            Some((ops, node_id))
                        })
                        .ok()
                        .flatten()
                };

                let Some((ops, node_id)) = planned else {
                    return;
                };

                self.apply_ops(cx.app, ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                    s.selected_nodes.push(node_id);
                    s.draw_order.retain(|id| *id != node_id);
                    s.draw_order.push(node_id);
                });
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::DeleteEdge) => {
                let remove_ops = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            graph
                                .edges
                                .get(edge_id)
                                .map(|edge| {
                                    vec![GraphOp::RemoveEdge {
                                        id: *edge_id,
                                        edge: edge.clone(),
                                    }]
                                })
                                .unwrap_or_default()
                        })
                        .ok()
                        .unwrap_or_default()
                };

                self.apply_ops(cx.app, remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| *id != *edge_id);
                });
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::Custom(action_id)) => {
                let ops = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.on_edge_context_menu_action(graph, *edge_id, action_id)
                        })
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                };

                if !ops.is_empty() {
                    self.apply_ops(cx.app, ops);
                }
            }
        }
    }

    fn paint_context_menu<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        menu: &ContextMenuState,
        zoom: f32,
    ) {
        let rect = context_menu_rect_at(&self.style, menu.origin, menu.items.len(), zoom);
        let border_w = Px(1.0 / zoom);
        let radius = Px(self.style.context_menu_corner_radius / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(50),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(border_w),
            border_color: self.style.context_menu_border,
            corner_radii: Corners::all(radius),
        });

        let pad = self.style.context_menu_padding / zoom;
        let item_h = self.style.context_menu_item_height / zoom;
        let inner_x = rect.origin.x.0 + pad;
        let inner_y = rect.origin.y.0 + pad;
        let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(inner_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        for (ix, item) in menu.items.iter().enumerate() {
            let item_rect = Rect::new(
                Point::new(Px(inner_x), Px(inner_y + ix as f32 * item_h)),
                Size::new(Px(inner_w), Px(item_h)),
            );

            if menu.hovered_item == Some(ix) && item.enabled {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(51),
                    rect: item_rect,
                    background: self.style.context_menu_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(4.0 / zoom)),
                });
            }

            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(item.label.as_ref(), &text_style, constraints);
            self.text_blobs.push(blob);

            let text_x = item_rect.origin.x;
            let inner_y =
                item_rect.origin.y.0 + (item_rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            let color = if item.enabled {
                self.style.context_menu_text
            } else {
                self.style.context_menu_text_disabled
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(52),
                origin: Point::new(text_x, text_y),
                text: blob,
                color,
            });
        }
    }

    fn yank_edge_from_port(
        &self,
        graph: &Graph,
        port: PortId,
    ) -> Option<(EdgeId, EdgeEndpoint, PortId)> {
        let p = graph.ports.get(&port)?;
        match p.dir {
            PortDirection::Out => {
                for (edge_id, edge) in &graph.edges {
                    if edge.from == port {
                        return Some((*edge_id, EdgeEndpoint::From, edge.to));
                    }
                }
            }
            PortDirection::In => {
                for (edge_id, edge) in &graph.edges {
                    if edge.to == port {
                        return Some((*edge_id, EdgeEndpoint::To, edge.from));
                    }
                }
            }
        }
        None
    }

    fn pick_reconnect_endpoint(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        pos: Point,
        zoom: f32,
    ) -> Option<(EdgeEndpoint, PortId)> {
        let edge = graph.edges.get(&edge_id)?;

        let from_center = {
            let from = graph.ports.get(&edge.from)?;
            self.port_center(graph, from.node, edge.from, zoom)
        };
        let to_center = {
            let to = graph.ports.get(&edge.to)?;
            self.port_center(graph, to.node, edge.to, zoom)
        };

        let (from_center, to_center) = match (from_center, to_center) {
            (Some(a), Some(b)) => (a, b),
            _ => return None,
        };

        let d2_from = {
            let dx = pos.x.0 - from_center.x.0;
            let dy = pos.y.0 - from_center.y.0;
            dx * dx + dy * dy
        };
        let d2_to = {
            let dx = pos.x.0 - to_center.x.0;
            let dy = pos.y.0 - to_center.y.0;
            dx * dx + dy * dy
        };

        if d2_from <= d2_to {
            Some((EdgeEndpoint::From, edge.to))
        } else {
            Some((EdgeEndpoint::To, edge.from))
        }
    }

    fn hit_node(
        &self,
        graph: &Graph,
        pos: Point,
        node_order: &[GraphNodeId],
        zoom: f32,
    ) -> Option<GraphNodeId> {
        for node in node_order.iter().rev() {
            let Some(rect) = self.node_rect(graph, *node, zoom) else {
                continue;
            };
            if rect.contains(pos) {
                return Some(*node);
            }
        }
        None
    }

    fn prepare_wire_path(
        services: &mut dyn fret_core::UiServices,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        width_px: f32,
    ) -> Option<fret_core::PathId> {
        let dx = to.x.0 - from.x.0;
        let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
        let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
        let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
        let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);

        let commands = [
            PathCommand::MoveTo(from),
            PathCommand::CubicTo {
                ctrl1: c1,
                ctrl2: c2,
                to,
            },
        ];

        let (id, _metrics) = services.path().prepare(
            &commands,
            PathStyle::Stroke(StrokeStyle {
                width: Px(width_px / zoom),
            }),
            PathConstraints {
                scale_factor: scale_factor * zoom,
            },
        );

        Some(id)
    }

    fn zoom_about_center(&mut self, bounds: Rect, delta_y: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }

        let speed = 0.0015;
        let factor = (1.0 + (-delta_y * speed)).clamp(0.2, 5.0);
        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let cx = 0.5 * bounds.size.width.0;
        let cy = 0.5 * bounds.size.height.0;
        let center_screen = (cx, cy);

        let pan_x = self.cached_pan.x;
        let pan_y = self.cached_pan.y;

        let g0_x = center_screen.0 / zoom - pan_x;
        let g0_y = center_screen.1 / zoom - pan_y;

        let new_pan_x = center_screen.0 / new_zoom - g0_x;
        let new_pan_y = center_screen.1 / new_zoom - g0_y;

        self.cached_pan = CanvasPoint {
            x: new_pan_x,
            y: new_pan_y,
        };
        self.cached_zoom = new_zoom;
    }
}

impl<H: UiHost> Widget<H> for NodeGraphCanvas {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.wire_paths.drain(..) {
            services.path().release(id);
        }
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn render_transform(&self, bounds: Rect) -> Option<Transform2D> {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return None;
        }
        let pan = Point::new(Px(self.cached_pan.x), Px(self.cached_pan.y));
        Some(
            Transform2D::translation(bounds.origin)
                .compose(Transform2D::scale_uniform(zoom))
                .compose(Transform2D::translation(pan)),
        )
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);
        self.sync_view_state(cx.app);
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let snapshot = self.sync_view_state(cx.app);
        let zoom = snapshot.zoom;

        match event {
            Event::KeyDown { key, .. } => {
                if *key == fret_core::KeyCode::Escape {
                    if self.interaction.context_menu.take().is_some() {
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    return;
                }

                if !matches!(
                    key,
                    fret_core::KeyCode::Delete | fret_core::KeyCode::Backspace
                ) {
                    return;
                }

                let selected_edges = snapshot.selected_edges.clone();
                if selected_edges.is_empty() {
                    return;
                }

                let remove_ops = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            let mut ops: Vec<GraphOp> = Vec::new();
                            for id in &selected_edges {
                                let Some(edge) = graph.edges.get(id) else {
                                    continue;
                                };
                                ops.push(GraphOp::RemoveEdge {
                                    id: *id,
                                    edge: edge.clone(),
                                });
                            }
                            ops
                        })
                        .ok()
                        .unwrap_or_default()
                };

                self.apply_ops(cx.app, remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                });
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.interaction.last_pos = Some(*position);

                if let Some(menu) = self.interaction.context_menu.as_mut() {
                    match button {
                        MouseButton::Left => {
                            if let Some(ix) =
                                hit_context_menu_item(&self.style, menu, *position, zoom)
                            {
                                let item = menu.items.get(ix).cloned();
                                let target = menu.target.clone();
                                let invoked_at = menu.invoked_at;
                                self.interaction.context_menu = None;
                                if let Some(item) = item
                                    && item.enabled
                                {
                                    self.activate_context_menu_item(cx, &target, invoked_at, item);
                                }
                            } else {
                                self.interaction.context_menu = None;
                            }
                            cx.stop_propagation();
                            cx.request_redraw();
                            cx.invalidate_self(Invalidation::Paint);
                            return;
                        }
                        MouseButton::Right => {
                            self.interaction.context_menu = None;
                        }
                        _ => {
                            self.interaction.context_menu = None;
                            cx.stop_propagation();
                            cx.request_redraw();
                            cx.invalidate_self(Invalidation::Paint);
                            return;
                        }
                    }
                }

                if *button == MouseButton::Middle {
                    self.interaction.hover_edge = None;
                    self.interaction.panning = true;
                    cx.capture_pointer(cx.node);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if *button == MouseButton::Right {
                    let hit_edge = {
                        let this = &*self;
                        this.graph
                            .read_ref(cx.app, |graph| {
                                this.hit_edge(graph, &snapshot, *position, zoom)
                            })
                            .ok()
                            .flatten()
                    };

                    let Some(edge) = hit_edge else {
                        return;
                    };

                    let items = {
                        let presenter = &mut *self.presenter;
                        let style = &self.style;
                        self.graph
                            .read_ref(cx.app, |graph| {
                                let mut items: Vec<NodeGraphContextMenuItem> = Vec::new();
                                presenter.fill_edge_context_menu(graph, edge, style, &mut items);
                                items.push(NodeGraphContextMenuItem {
                                    label: Arc::<str>::from("Insert Reroute"),
                                    enabled: true,
                                    action: NodeGraphContextMenuAction::InsertReroute,
                                });
                                items.push(NodeGraphContextMenuItem {
                                    label: Arc::<str>::from("Delete"),
                                    enabled: true,
                                    action: NodeGraphContextMenuAction::DeleteEdge,
                                });
                                items
                            })
                            .ok()
                            .unwrap_or_default()
                    };

                    let origin = self.clamp_context_menu_origin(
                        *position,
                        items.len(),
                        cx.bounds,
                        &snapshot,
                    );
                    self.interaction.context_menu = Some(ContextMenuState {
                        origin,
                        invoked_at: *position,
                        target: ContextMenuTarget::Edge(edge),
                        items,
                        hovered_item: None,
                    });
                    self.interaction.hover_edge = None;

                    self.update_view_state(cx.app, |s| {
                        s.selected_nodes.clear();
                        if !s.selected_edges.iter().any(|id| *id == edge) {
                            s.selected_edges.clear();
                            s.selected_edges.push(edge);
                        }
                    });

                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if *button != MouseButton::Left {
                    return;
                }

                self.interaction.hover_edge = None;

                #[derive(Debug, Clone, Copy)]
                enum Hit {
                    Port(PortId),
                    Node(GraphNodeId, Rect),
                    Edge(EdgeId),
                    Background,
                }

                let hit = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            if let Some(port) = this.hit_port(graph, *position, zoom) {
                                return Hit::Port(port);
                            }
                            let order = this.node_order(graph, &snapshot);
                            let Some(node) = this.hit_node(graph, *position, &order, zoom) else {
                                if let Some(edge) = this.hit_edge(graph, &snapshot, *position, zoom)
                                {
                                    return Hit::Edge(edge);
                                }
                                return Hit::Background;
                            };
                            let Some(rect) = this.node_rect(graph, node, zoom) else {
                                return Hit::Background;
                            };
                            Hit::Node(node, rect)
                        })
                        .unwrap_or(Hit::Background)
                };

                match hit {
                    Hit::Port(port) => {
                        self.interaction.edge_drag = None;
                        let yank = (modifiers.ctrl || modifiers.meta).then(|| {
                            let this = &*self;
                            this.graph
                                .read_ref(cx.app, |graph| this.yank_edge_from_port(graph, port))
                                .ok()
                                .flatten()
                        });

                        let kind = match yank.flatten() {
                            Some((edge, endpoint, fixed)) => WireDragKind::Reconnect {
                                edge,
                                endpoint,
                                fixed,
                            },
                            None => WireDragKind::New { from: port },
                        };

                        self.interaction.wire_drag = Some(WireDrag {
                            kind,
                            pos: *position,
                        });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Node(node, rect) => {
                        self.interaction.edge_drag = None;
                        let offset = Point::new(
                            Px(position.x.0 - rect.origin.x.0),
                            Px(position.y.0 - rect.origin.y.0),
                        );
                        self.interaction.node_drag = Some(NodeDrag {
                            node,
                            grab_offset: offset,
                        });
                        cx.capture_pointer(cx.node);

                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
                            s.selected_edges.clear();
                            s.selected_nodes.push(node);
                            s.draw_order.retain(|id| *id != node);
                            s.draw_order.push(node);
                        });

                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Edge(edge) => {
                        let multi = modifiers.ctrl || modifiers.meta;
                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
                            if multi {
                                if let Some(ix) = s.selected_edges.iter().position(|id| *id == edge)
                                {
                                    s.selected_edges.remove(ix);
                                } else {
                                    s.selected_edges.push(edge);
                                }
                            } else {
                                s.selected_edges.clear();
                                s.selected_edges.push(edge);
                            }
                        });
                        self.interaction.edge_drag = Some(EdgeDrag {
                            edge,
                            start_pos: *position,
                        });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Background => {
                        self.interaction.edge_drag = None;
                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
                            s.selected_edges.clear();
                        });
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let Some(last) = self.interaction.last_pos else {
                    self.interaction.last_pos = Some(*position);
                    return;
                };
                let delta = Point::new(Px(position.x.0 - last.x.0), Px(position.y.0 - last.y.0));
                self.interaction.last_pos = Some(*position);

                if self.interaction.panning {
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += delta.x.0;
                        s.pan.y += delta.y.0;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if let Some(drag) = &self.interaction.node_drag {
                    let new_pos = Point::new(
                        Px(position.x.0 - drag.grab_offset.x.0),
                        Px(position.y.0 - drag.grab_offset.y.0),
                    );
                    let id = drag.node;
                    let _ = self.graph.update(cx.app, |g, _cx| {
                        let Some(node) = g.nodes.get(&id) else {
                            return;
                        };
                        let from = node.pos;
                        let to = CanvasPoint {
                            x: new_pos.x.0,
                            y: new_pos.y.0,
                        };
                        let tx = GraphTransaction {
                            label: None,
                            ops: vec![GraphOp::SetNodePos { id, from, to }],
                        };
                        let _ = apply_transaction(g, &tx);
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if let Some(w) = &mut self.interaction.wire_drag {
                    w.pos = *position;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if let Some(drag) = self.interaction.edge_drag.clone() {
                    let threshold = (3.0 / zoom).max(0.5 / zoom);
                    let dx = position.x.0 - drag.start_pos.x.0;
                    let dy = position.y.0 - drag.start_pos.y.0;
                    if dx * dx + dy * dy >= threshold * threshold {
                        let reconnect = {
                            let this = &*self;
                            this.graph
                                .read_ref(cx.app, |graph| {
                                    this.pick_reconnect_endpoint(
                                        graph,
                                        drag.edge,
                                        drag.start_pos,
                                        zoom,
                                    )
                                })
                                .ok()
                                .flatten()
                        };

                        if let Some((endpoint, fixed)) = reconnect {
                            self.interaction.edge_drag = None;
                            self.interaction.hover_edge = None;
                            self.interaction.wire_drag = Some(WireDrag {
                                kind: WireDragKind::Reconnect {
                                    edge: drag.edge,
                                    endpoint,
                                    fixed,
                                },
                                pos: *position,
                            });

                            cx.request_redraw();
                            cx.invalidate_self(Invalidation::Paint);
                            return;
                        }
                    }
                }

                if let Some(menu) = self.interaction.context_menu.as_mut() {
                    let new_hover = hit_context_menu_item(&self.style, menu, *position, zoom);
                    if menu.hovered_item != new_hover {
                        menu.hovered_item = new_hover;
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    return;
                }

                let new_hover = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            this.hit_edge(graph, &snapshot, *position, zoom)
                        })
                        .ok()
                        .flatten()
                };

                if self.interaction.hover_edge != new_hover {
                    self.interaction.hover_edge = new_hover;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                self.interaction.last_pos = Some(*position);

                if *button == MouseButton::Middle && self.interaction.panning {
                    self.interaction.panning = false;
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if *button == MouseButton::Left {
                    if self.interaction.node_drag.is_some() {
                        self.interaction.node_drag = None;
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }

                    if let Some(w) = self.interaction.wire_drag.take() {
                        let target = {
                            let this = &*self;
                            this.graph
                                .read_ref(cx.app, |graph| this.hit_port(graph, *position, zoom))
                                .ok()
                                .flatten()
                        };

                        match w.kind {
                            WireDragKind::New { from } => {
                                if let Some(target) = target
                                    && target != from
                                {
                                    let plan_ops = {
                                        let presenter = &mut *self.presenter;
                                        self.graph
                                            .read_ref(cx.app, |graph| {
                                                let plan =
                                                    presenter.plan_connect(graph, from, target);
                                                (plan.decision
                                                    == crate::rules::ConnectDecision::Accept)
                                                    .then_some(plan.ops)
                                            })
                                            .ok()
                                            .flatten()
                                    };
                                    if let Some(ops) = plan_ops {
                                        self.apply_ops(cx.app, ops);
                                    }
                                }
                            }
                            WireDragKind::Reconnect {
                                edge,
                                endpoint,
                                fixed: _,
                            } => {
                                if let Some(target) = target {
                                    let plan_ops = {
                                        let presenter = &mut *self.presenter;
                                        self.graph
                                            .read_ref(cx.app, |graph| {
                                                let plan = presenter.plan_reconnect_edge(
                                                    graph, edge, endpoint, target,
                                                );
                                                (plan.decision
                                                    == crate::rules::ConnectDecision::Accept)
                                                    .then_some(plan.ops)
                                            })
                                            .ok()
                                            .flatten()
                                    };
                                    if let Some(ops) = plan_ops {
                                        self.apply_ops(cx.app, ops);
                                    }
                                }
                            }
                        }
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }

                    if self.interaction.edge_drag.take().is_some() {
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                }
            }
            Event::Pointer(fret_core::PointerEvent::Wheel {
                delta, modifiers, ..
            }) => {
                if modifiers.ctrl || modifiers.meta {
                    let delta_screen_y = delta.y.0 * zoom;
                    self.zoom_about_center(cx.bounds, delta_screen_y);
                    let pan = self.cached_pan;
                    let zoom = self.cached_zoom;
                    self.update_view_state(cx.app, |s| {
                        s.pan = pan;
                        s.zoom = zoom;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                } else {
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += delta.x.0;
                        s.pan.y += delta.y.0;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        let snapshot = self.sync_view_state(cx.app);

        for id in self.wire_paths.drain(..) {
            cx.services.path().release(id);
        }
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let zoom = snapshot.zoom;
        let pan = snapshot.pan;

        let viewport_w = cx.bounds.size.width.0 / zoom;
        let viewport_h = cx.bounds.size.height.0 / zoom;
        let viewport_origin_x = -pan.x;
        let viewport_origin_y = -pan.y;

        cx.scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(viewport_origin_x), Px(viewport_origin_y)),
                Size::new(Px(viewport_w), Px(viewport_h)),
            ),
        });

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(
                Point::new(Px(viewport_origin_x), Px(viewport_origin_y)),
                Size::new(Px(viewport_w), Px(viewport_h)),
            ),
            background: self.style.background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        let spacing = self.style.grid_spacing;
        if spacing.is_finite() && spacing > 1.0e-3 {
            let major_every = self.style.grid_major_every.max(1) as i64;
            let thickness = Px((1.0 / zoom).max(0.25 / zoom));

            let x0 = (viewport_origin_x / spacing).floor() as i64;
            let x1 = ((viewport_origin_x + viewport_w) / spacing).ceil() as i64;
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let color = if ix.rem_euclid(major_every) == 0 {
                    self.style.grid_major_color
                } else {
                    self.style.grid_minor_color
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(x - 0.5 * thickness.0), Px(viewport_origin_y)),
                        Size::new(thickness, Px(viewport_h)),
                    ),
                    background: color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let y0 = (viewport_origin_y / spacing).floor() as i64;
            let y1 = ((viewport_origin_y + viewport_h) / spacing).ceil() as i64;
            for iy in y0..=y1 {
                let y = iy as f32 * spacing;
                let color = if iy.rem_euclid(major_every) == 0 {
                    self.style.grid_major_color
                } else {
                    self.style.grid_minor_color
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(viewport_origin_x), Px(y - 0.5 * thickness.0)),
                        Size::new(Px(viewport_w), thickness),
                    ),
                    background: color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }

        #[derive(Debug, Default)]
        struct RenderData {
            edges: Vec<(EdgeId, Point, Point, Color, bool, bool)>,
            nodes: Vec<(GraphNodeId, Rect, bool)>,
            pins: Vec<(Rect, Color)>,
            port_centers: HashMap<PortId, Point>,
        }

        let hovered_edge = self.interaction.hover_edge;

        let render = {
            let selected: HashSet<GraphNodeId> = snapshot.selected_nodes.iter().copied().collect();
            let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();
            let skip_edge = self
                .interaction
                .wire_drag
                .as_ref()
                .and_then(|w| match w.kind {
                    WireDragKind::Reconnect { edge, .. } => Some(edge),
                    _ => None,
                });
            let this = &*self;
            let presenter: &dyn NodeGraphPresenter = &*this.presenter;
            this.graph
                .read_ref(cx.app, |graph| {
                    let mut out = RenderData::default();

                    let node_order = this.node_order(graph, &snapshot);
                    for node in &node_order {
                        let Some(rect) = this.node_rect(graph, *node, zoom) else {
                            continue;
                        };
                        let is_selected = selected.contains(node);
                        out.nodes.push((*node, rect, is_selected));

                        let (inputs, outputs) = this.node_ports(graph, *node);
                        for port in inputs.iter().chain(outputs.iter()) {
                            if let Some(center) = this.port_center(graph, *node, *port, zoom) {
                                out.port_centers.insert(*port, center);
                                let color = presenter.port_color(graph, *port, &this.style);
                                let pin_r = this.style.pin_radius / zoom;
                                let rect = Rect::new(
                                    Point::new(Px(center.x.0 - pin_r), Px(center.y.0 - pin_r)),
                                    Size::new(Px(2.0 * pin_r), Px(2.0 * pin_r)),
                                );
                                out.pins.push((rect, color));
                            }
                        }
                    }

                    for (&edge_id, edge) in &graph.edges {
                        if skip_edge == Some(edge_id) {
                            continue;
                        }
                        let Some(from) = out.port_centers.get(&edge.from).copied() else {
                            continue;
                        };
                        let Some(to) = out.port_centers.get(&edge.to).copied() else {
                            continue;
                        };
                        let color = presenter.edge_color(graph, edge_id, &this.style);
                        out.edges.push((
                            edge_id,
                            from,
                            to,
                            color,
                            selected_edges.contains(&edge_id),
                            hovered_edge == Some(edge_id),
                        ));
                    }

                    out
                })
                .unwrap_or_default()
        };

        let mut edges_normal: Vec<(Point, Point, Color, f32)> = Vec::new();
        let mut edges_selected: Vec<(Point, Point, Color, f32)> = Vec::new();
        let mut edges_hovered: Vec<(Point, Point, Color, f32)> = Vec::new();

        for (_edge_id, from, to, color, selected, hovered) in render.edges {
            let mut width = self.style.wire_width;
            if selected {
                width *= self.style.wire_width_selected_mul;
            }
            if hovered {
                width *= self.style.wire_width_hover_mul;
            }

            if hovered {
                edges_hovered.push((from, to, color, width));
            } else if selected {
                edges_selected.push((from, to, color, width));
            } else {
                edges_normal.push((from, to, color, width));
            }
        }

        for (from, to, color, width) in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            if let Some(path) =
                Self::prepare_wire_path(cx.services, from, to, zoom, cx.scale_factor, width)
            {
                self.wire_paths.push(path);
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
        }

        if let Some(w) = &self.interaction.wire_drag {
            let from = match &w.kind {
                WireDragKind::New { from } => render.port_centers.get(from).copied(),
                WireDragKind::Reconnect { fixed, .. } => render.port_centers.get(fixed).copied(),
            };
            if let Some(from) = from {
                let to = w.pos;
                let color = self.style.wire_color_preview;
                if let Some(path) = Self::prepare_wire_path(
                    cx.services,
                    from,
                    to,
                    zoom,
                    cx.scale_factor,
                    self.style.wire_width,
                ) {
                    self.wire_paths.push(path);
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }
        }

        let corner = Px(8.0 / zoom);
        for (_node, rect, is_selected) in render.nodes {
            let border_color = if is_selected {
                self.style.node_border_selected
            } else {
                self.style.node_border
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: self.style.node_background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color,
                corner_radii: Corners::all(corner),
            });
        }

        for (rect, color) in render.pins {
            let r = Px(0.5 * rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect,
                background: color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(r),
            });
        }

        if let Some(menu) = self.interaction.context_menu.clone() {
            self.paint_context_menu(cx, &menu, zoom);
        }

        cx.scene.push(SceneOp::PopClip);
    }
}

fn context_menu_rect_at(
    style: &NodeGraphStyle,
    origin: Point,
    item_count: usize,
    zoom: f32,
) -> Rect {
    let w = style.context_menu_width / zoom;
    let item_h = style.context_menu_item_height / zoom;
    let pad = style.context_menu_padding / zoom;
    let h = (2.0 * pad + item_h * item_count.max(1) as f32).max(item_h + 2.0 * pad);
    Rect::new(origin, Size::new(Px(w), Px(h)))
}

fn hit_context_menu_item(
    style: &NodeGraphStyle,
    menu: &ContextMenuState,
    pos: Point,
    zoom: f32,
) -> Option<usize> {
    let rect = context_menu_rect_at(style, menu.origin, menu.items.len(), zoom);
    if !rect.contains(pos) {
        return None;
    }

    let pad = style.context_menu_padding / zoom;
    let item_h = style.context_menu_item_height / zoom;
    let inner_top = rect.origin.y.0 + pad;
    let y = pos.y.0 - inner_top;
    if y < 0.0 {
        return None;
    }

    let ix = (y / item_h).floor() as isize;
    if ix < 0 {
        return None;
    }
    let ix = ix as usize;
    (ix < menu.items.len()).then_some(ix)
}

fn wire_distance2(p: Point, from: Point, to: Point, zoom: f32) -> f32 {
    let (c1, c2) = wire_ctrl_points(from, to, zoom);

    let steps: usize = 24;
    let mut best = f32::INFINITY;

    let mut prev = from;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let cur = cubic_bezier(from, c1, c2, to, t);
        best = best.min(dist2_point_to_segment(p, prev, cur));
        prev = cur;
    }

    best
}

fn wire_ctrl_points(from: Point, to: Point, zoom: f32) -> (Point, Point) {
    let dx = to.x.0 - from.x.0;
    let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
    let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
    let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
    let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);
    (c1, c2)
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    let w0 = mt2 * mt;
    let w1 = 3.0 * mt2 * t;
    let w2 = 3.0 * mt * t2;
    let w3 = t2 * t;

    Point::new(
        Px(w0 * p0.x.0 + w1 * p1.x.0 + w2 * p2.x.0 + w3 * p3.x.0),
        Px(w0 * p0.y.0 + w1 * p1.y.0 + w2 * p2.y.0 + w3 * p3.y.0),
    )
}

fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if ab2 <= 1.0e-9 {
        return apx * apx + apy * apy;
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    dx * dx + dy * dy
}
