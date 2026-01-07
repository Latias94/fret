use std::collections::{HashMap, HashSet};

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, PathCommand, PathConstraints, PathStyle,
    Point, Px, Rect, SceneOp, Size, StrokeStyle, Transform2D,
};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::{CanvasPoint, EdgeId, Graph, NodeId as GraphNodeId, PortDirection, PortId};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::rules::EdgeEndpoint;

use super::presenter::{DefaultNodeGraphPresenter, NodeGraphPresenter};
use super::style::NodeGraphStyle;

#[derive(Debug, Clone)]
struct ViewSnapshot {
    pan: CanvasPoint,
    zoom: f32,
    selected_nodes: Vec<GraphNodeId>,
    draw_order: Vec<GraphNodeId>,
}

#[derive(Debug, Default, Clone)]
struct InteractionState {
    last_pos: Option<Point>,
    panning: bool,
    node_drag: Option<NodeDrag>,
    wire_drag: Option<WireDrag>,
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
            draw_order: Vec::new(),
        };

        let _ = self.view_state.read(host, |_host, s| {
            snapshot.pan = s.pan;
            snapshot.zoom = s.zoom;
            snapshot.selected_nodes = s.selected_nodes.clone();
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
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.interaction.last_pos = Some(*position);

                if *button == MouseButton::Middle {
                    self.interaction.panning = true;
                    cx.capture_pointer(cx.node);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if *button != MouseButton::Left {
                    return;
                }

                #[derive(Debug, Clone, Copy)]
                enum Hit {
                    Port(PortId),
                    Node(GraphNodeId, Rect),
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
                            s.selected_nodes.push(node);
                            s.draw_order.retain(|id| *id != node);
                            s.draw_order.push(node);
                        });

                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Background => {
                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
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
            edges: Vec<(Point, Point, Color)>,
            nodes: Vec<(GraphNodeId, Rect, bool)>,
            pins: Vec<(Rect, Color)>,
            port_centers: HashMap<PortId, Point>,
        }

        let render = {
            let selected: HashSet<GraphNodeId> = snapshot.selected_nodes.iter().copied().collect();
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
                        out.edges.push((from, to, color));
                    }

                    out
                })
                .unwrap_or_default()
        };

        for (from, to, color) in render.edges {
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

        cx.scene.push(SceneOp::PopClip);
    }
}
