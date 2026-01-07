use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, MouseButton, PathCommand,
    PathConstraints, PathStyle, Point, Px, Rect, SceneOp, Size, StrokeStyle, TextBlobId,
    TextConstraints, TextOverflow, TextWrap, Transform2D,
};
use fret_runtime::{CommandId, Effect, Model, TimerToken};
use fret_ui::{UiHost, retained_bridge::*};

use crate::REROUTE_KIND;
use crate::core::{
    CanvasPoint, CanvasSize, EdgeId, Graph, NodeId as GraphNodeId, NodeKindKey, PortDirection,
    PortId,
};
use crate::io::{NodeGraphConnectionMode, NodeGraphInteractionState, NodeGraphViewState};
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::rules::{ConnectDecision, Diagnostic, DiagnosticSeverity, EdgeEndpoint};

use crate::ui::presenter::{
    DefaultNodeGraphPresenter, InsertNodeCandidate, NodeGraphContextMenuAction,
    NodeGraphContextMenuItem, NodeGraphPresenter,
};
use crate::ui::style::NodeGraphStyle;

use super::conversion;

#[derive(Debug, Clone)]
struct ViewSnapshot {
    pan: CanvasPoint,
    zoom: f32,
    selected_nodes: Vec<GraphNodeId>,
    selected_edges: Vec<EdgeId>,
    draw_order: Vec<GraphNodeId>,
    interaction: NodeGraphInteractionState,
}

#[derive(Debug, Default, Clone)]
struct InteractionState {
    last_pos: Option<Point>,
    panning: bool,
    pending_node_drag: Option<PendingNodeDrag>,
    node_drag: Option<NodeDrag>,
    wire_drag: Option<WireDrag>,
    edge_drag: Option<EdgeDrag>,
    hover_edge: Option<EdgeId>,
    hover_port: Option<PortId>,
    hover_port_valid: bool,
    hover_port_convertible: bool,
    context_menu: Option<ContextMenuState>,
    toast: Option<ToastState>,
}

#[derive(Debug, Clone)]
struct PendingNodeDrag {
    node: GraphNodeId,
    grab_offset: Point,
    start_pos: Point,
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
        bundle: Vec<PortId>,
    },
    Reconnect {
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        fixed: PortId,
    },
    ReconnectMany {
        edges: Vec<(EdgeId, EdgeEndpoint, PortId)>,
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
    EdgeInsertNodePicker(EdgeId),
    ConnectionConvertPicker {
        from: PortId,
        to: PortId,
        at: CanvasPoint,
    },
}

#[derive(Debug, Clone)]
struct ContextMenuState {
    origin: Point,
    invoked_at: Point,
    target: ContextMenuTarget,
    items: Vec<NodeGraphContextMenuItem>,
    candidates: Vec<InsertNodeCandidate>,
    hovered_item: Option<usize>,
    active_item: usize,
    typeahead: String,
}

#[derive(Debug, Clone)]
struct ToastState {
    timer: TimerToken,
    severity: DiagnosticSeverity,
    message: Arc<str>,
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
    close_command: Option<CommandId>,

    cached_pan: CanvasPoint,
    cached_zoom: f32,

    wire_paths: Vec<fret_core::PathId>,
    text_blobs: Vec<TextBlobId>,
    interaction: InteractionState,
}

impl NodeGraphCanvas {
    const REROUTE_INPUTS: usize = 1;
    const REROUTE_OUTPUTS: usize = 1;
    const AUTO_PAN_TICK_HZ: f32 = 60.0;

    fn show_toast<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        severity: DiagnosticSeverity,
        message: impl Into<Arc<str>>,
    ) {
        if let Some(prev) = self.interaction.toast.take() {
            host.push_effect(Effect::CancelTimer { token: prev.timer });
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Duration::from_millis(2400),
            repeat: None,
        });

        self.interaction.toast = Some(ToastState {
            timer,
            severity,
            message: message.into(),
        });
    }

    fn toast_from_diagnostics(diags: &[Diagnostic]) -> Option<(DiagnosticSeverity, Arc<str>)> {
        let first = diags.first()?;
        if first.message.is_empty() {
            return None;
        }
        Some((first.severity, Arc::<str>::from(first.message.clone())))
    }

    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        Self {
            graph,
            view_state,
            presenter: Box::new(DefaultNodeGraphPresenter::default()),
            style: NodeGraphStyle::default(),
            close_command: None,
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

    /// Adds a screen-space close button overlay that dispatches a command when clicked.
    ///
    /// This is intended for demos and tool windows; production apps typically wire close actions
    /// via docking/tab chrome instead.
    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = Some(command);
        self
    }

    fn close_button_rect(pan: CanvasPoint, zoom: f32) -> Rect {
        let margin = 12.0 / zoom;
        let w = 64.0 / zoom;
        let h = 24.0 / zoom;
        Rect::new(
            Point::new(Px(-pan.x + margin), Px(-pan.y + margin)),
            Size::new(Px(w), Px(h)),
        )
    }

    fn rect_contains(rect: Rect, pos: Point) -> bool {
        pos.x.0 >= rect.origin.x.0
            && pos.y.0 >= rect.origin.y.0
            && pos.x.0 <= rect.origin.x.0 + rect.size.width.0
            && pos.y.0 <= rect.origin.y.0 + rect.size.height.0
    }

    fn sync_view_state<H: UiHost>(&mut self, host: &mut H) -> ViewSnapshot {
        let mut snapshot = ViewSnapshot {
            pan: self.cached_pan,
            zoom: self.cached_zoom,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            draw_order: Vec::new(),
            interaction: NodeGraphInteractionState::default(),
        };

        let _ = self.view_state.read(host, |_host, s| {
            snapshot.pan = s.pan;
            snapshot.zoom = s.zoom;
            snapshot.selected_nodes = s.selected_nodes.clone();
            snapshot.selected_edges = s.selected_edges.clone();
            snapshot.draw_order = s.draw_order.clone();
            snapshot.interaction = s.interaction.clone();
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

    fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
        fn snap_axis(value: f32, grid: f32) -> f32 {
            if !value.is_finite() {
                return value;
            }
            if !grid.is_finite() || grid <= 0.0 {
                return value;
            }
            (value / grid).round() * grid
        }

        CanvasPoint {
            x: snap_axis(pos.x, grid.width),
            y: snap_axis(pos.y, grid.height),
        }
    }

    fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return CanvasPoint::default();
        }

        let margin_screen = snapshot.interaction.auto_pan.margin;
        let speed_screen_per_s = snapshot.interaction.auto_pan.speed;
        if !margin_screen.is_finite() || margin_screen <= 0.0 {
            return CanvasPoint::default();
        }
        if !speed_screen_per_s.is_finite() || speed_screen_per_s <= 0.0 {
            return CanvasPoint::default();
        }

        let viewport_w = bounds.size.width.0;
        let viewport_h = bounds.size.height.0;
        if !viewport_w.is_finite()
            || viewport_w <= 0.0
            || !viewport_h.is_finite()
            || viewport_h <= 0.0
        {
            return CanvasPoint::default();
        }

        let pan = snapshot.pan;
        let pos_screen_x = (pos.x.0 + pan.x) * zoom;
        let pos_screen_y = (pos.y.0 + pan.y) * zoom;

        let dist_left = pos_screen_x;
        let dist_right = viewport_w - pos_screen_x;
        let dist_top = pos_screen_y;
        let dist_bottom = viewport_h - pos_screen_y;

        let step_screen = speed_screen_per_s / Self::AUTO_PAN_TICK_HZ;
        let step_graph = step_screen / zoom;

        let mut delta_x = 0.0;
        let mut delta_y = 0.0;

        if dist_left.is_finite() && dist_left < margin_screen {
            let factor = ((margin_screen - dist_left) / margin_screen).clamp(0.0, 1.0);
            delta_x += step_graph * factor;
        }
        if dist_right.is_finite() && dist_right < margin_screen {
            let factor = ((margin_screen - dist_right) / margin_screen).clamp(0.0, 1.0);
            delta_x -= step_graph * factor;
        }
        if dist_top.is_finite() && dist_top < margin_screen {
            let factor = ((margin_screen - dist_top) / margin_screen).clamp(0.0, 1.0);
            delta_y += step_graph * factor;
        }
        if dist_bottom.is_finite() && dist_bottom < margin_screen {
            let factor = ((margin_screen - dist_bottom) / margin_screen).clamp(0.0, 1.0);
            delta_y -= step_graph * factor;
        }

        if !delta_x.is_finite() || !delta_y.is_finite() {
            return CanvasPoint::default();
        }

        CanvasPoint {
            x: delta_x,
            y: delta_y,
        }
    }

    fn wire_drag_suppresses_edge(kind: &WireDragKind, edge_id: EdgeId) -> bool {
        match kind {
            WireDragKind::Reconnect { edge, .. } => *edge == edge_id,
            WireDragKind::ReconnectMany { edges } => {
                edges.iter().any(|(edge, ..)| *edge == edge_id)
            }
            _ => false,
        }
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

    fn pick_target_port(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        from: PortId,
        pos: Point,
        zoom: f32,
    ) -> Option<PortId> {
        let from_port = graph.ports.get(&from)?;
        let desired_dir = match from_port.dir {
            PortDirection::In => PortDirection::Out,
            PortDirection::Out => PortDirection::In,
        };

        match snapshot.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => {
                let candidate = self.hit_port(graph, pos, zoom)?;
                let port = graph.ports.get(&candidate)?;
                (candidate != from && port.dir == desired_dir).then_some(candidate)
            }
            NodeGraphConnectionMode::Loose => {
                let radius_screen = snapshot.interaction.connection_radius;
                if !radius_screen.is_finite() || radius_screen <= 0.0 {
                    let candidate = self.hit_port(graph, pos, zoom)?;
                    let port = graph.ports.get(&candidate)?;
                    return (candidate != from && port.dir == desired_dir).then_some(candidate);
                }
                let r = radius_screen / zoom;
                let r2 = r * r;

                let mut best: Option<(PortId, f32)> = None;
                for (&port_id, port) in &graph.ports {
                    if port_id == from || port.dir != desired_dir {
                        continue;
                    }
                    let Some(center) = self.port_center(graph, port.node, port_id, zoom) else {
                        continue;
                    };
                    let dx = center.x.0 - pos.x.0;
                    let dy = center.y.0 - pos.y.0;
                    let d2 = dx * dx + dy * dy;
                    if d2 > r2 {
                        continue;
                    }
                    match best {
                        Some((_id, best_d2)) if best_d2 <= d2 => {}
                        _ => best = Some((port_id, d2)),
                    }
                }

                best.map(|(id, _)| id)
            }
        }
    }

    fn hit_edge(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        pos: Point,
        zoom: f32,
    ) -> Option<EdgeId> {
        let hit_w =
            (snapshot.interaction.edge_interaction_width / zoom).max(self.style.wire_width / zoom);
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

    fn node_default_size_for_ports(&self, inputs: usize, outputs: usize) -> (f32, f32) {
        let rows = inputs.max(outputs) as f32;
        let base = self.style.node_header_height + 2.0 * self.style.node_padding;
        let pin_area = rows * self.style.pin_row_height;
        (self.style.node_width, base + pin_area)
    }

    fn reroute_pos_for_invoked_at(&self, invoked_at: Point) -> CanvasPoint {
        let (w, h) = self.node_default_size_for_ports(Self::REROUTE_INPUTS, Self::REROUTE_OUTPUTS);
        CanvasPoint {
            x: invoked_at.x.0 - 0.5 * w,
            y: invoked_at.y.0 - 0.5 * h,
        }
    }

    fn first_added_node_id(ops: &[GraphOp]) -> Option<GraphNodeId> {
        for op in ops {
            if let GraphOp::AddNode { id, .. } = op {
                return Some(*id);
            }
        }
        None
    }

    fn activate_context_menu_item<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    ) {
        match (target, item.action) {
            (
                ContextMenuTarget::Edge(edge_id),
                NodeGraphContextMenuAction::OpenInsertNodePicker,
            ) => {
                let candidates: Vec<InsertNodeCandidate> = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.list_insertable_nodes_for_edge(graph, *edge_id)
                        })
                        .ok()
                        .unwrap_or_default()
                };

                let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
                menu_candidates.push(InsertNodeCandidate {
                    kind: NodeKindKey::new(REROUTE_KIND),
                    label: Arc::<str>::from("Reroute"),
                    enabled: true,
                    template: None,
                    payload: serde_json::Value::Null,
                });
                menu_candidates.extend(candidates);

                let mut items: Vec<NodeGraphContextMenuItem> = Vec::new();
                for (ix, c) in menu_candidates.iter().enumerate() {
                    items.push(NodeGraphContextMenuItem {
                        label: c.label.clone(),
                        enabled: c.enabled,
                        action: NodeGraphContextMenuAction::InsertNodeCandidate(ix),
                    });
                }

                let snapshot = self.sync_view_state(cx.app);
                let origin =
                    self.clamp_context_menu_origin(invoked_at, items.len(), cx.bounds, &snapshot);

                let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
                self.interaction.context_menu = Some(ContextMenuState {
                    origin,
                    invoked_at,
                    target: ContextMenuTarget::EdgeInsertNodePicker(*edge_id),
                    items,
                    candidates: menu_candidates,
                    hovered_item: None,
                    active_item,
                    typeahead: String::new(),
                });
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::InsertReroute) => {
                let at = self.reroute_pos_for_invoked_at(invoked_at);
                let kind = NodeKindKey::new(REROUTE_KIND);

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter.plan_split_edge(graph, *edge_id, &kind, at);
                            match plan.decision {
                                ConnectDecision::Accept => Ok(plan.ops),
                                ConnectDecision::Reject => Err(plan.diagnostics),
                            }
                        })
                        .ok()
                };

                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        self.apply_ops(cx.app, ops);
                        if let Some(node_id) = node_id {
                            self.update_view_state(cx.app, |s| {
                                s.selected_edges.clear();
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node_id);
                                s.draw_order.retain(|id| *id != node_id);
                                s.draw_order.push(node_id);
                            });
                        }
                    }
                    Some(Err(diags)) => {
                        let (sev, msg) =
                            Self::toast_from_diagnostics(&diags).unwrap_or_else(|| {
                                (
                                    DiagnosticSeverity::Error,
                                    Arc::<str>::from("failed to insert reroute"),
                                )
                            });
                        self.show_toast(cx.app, cx.window, sev, msg);
                    }
                    None => {}
                }
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
            (
                ContextMenuTarget::EdgeInsertNodePicker(edge_id),
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };

                let outcome = {
                    let at = if candidate.kind.0 == REROUTE_KIND {
                        self.reroute_pos_for_invoked_at(invoked_at)
                    } else {
                        CanvasPoint {
                            x: invoked_at.x.0,
                            y: invoked_at.y.0,
                        }
                    };

                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter
                                .plan_split_edge_candidate(graph, *edge_id, &candidate, at);
                            match plan.decision {
                                ConnectDecision::Accept => Outcome::Apply(plan.ops),
                                ConnectDecision::Reject => {
                                    Self::toast_from_diagnostics(&plan.diagnostics)
                                        .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                        .unwrap_or_else(|| {
                                            Outcome::Reject(
                                                DiagnosticSeverity::Error,
                                                Arc::<str>::from(format!(
                                                    "node insertion was rejected: {}",
                                                    candidate.kind.0
                                                )),
                                            )
                                        })
                                }
                            }
                        })
                        .ok()
                        .unwrap_or(Outcome::Ignore)
                };

                match outcome {
                    Outcome::Apply(ops) => {
                        let select_node = candidate.kind.0 == REROUTE_KIND;
                        let node_id = select_node
                            .then(|| Self::first_added_node_id(&ops))
                            .flatten();
                        self.apply_ops(cx.app, ops);
                        if let Some(node_id) = node_id {
                            self.update_view_state(cx.app, |s| {
                                s.selected_edges.clear();
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node_id);
                                s.draw_order.retain(|id| *id != node_id);
                                s.draw_order.push(node_id);
                            });
                        }
                    }
                    Outcome::Reject(sev, msg) => self.show_toast(cx.app, cx.window, sev, msg),
                    Outcome::Ignore => {}
                }
            }
            (
                ContextMenuTarget::ConnectionConvertPicker { from, to, at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };

                let zoom = self.cached_zoom;
                let style = self.style.clone();

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let template = match &candidate.template {
                                Some(t) => t,
                                None => return Outcome::Ignore,
                            };

                            let plan = conversion::plan_insert_conversion(
                                presenter, graph, &style, zoom, *from, *to, *at, template,
                            );
                            match plan.decision {
                                ConnectDecision::Accept => Outcome::Apply(plan.ops),
                                ConnectDecision::Reject => {
                                    Self::toast_from_diagnostics(&plan.diagnostics)
                                        .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                        .unwrap_or(Outcome::Ignore)
                                }
                            }
                        })
                        .ok()
                        .unwrap_or(Outcome::Ignore)
                };

                match outcome {
                    Outcome::Apply(ops) => {
                        let node_id = Self::first_added_node_id(&ops);
                        self.apply_ops(cx.app, ops);
                        if let Some(node_id) = node_id {
                            self.update_view_state(cx.app, |s| {
                                s.selected_edges.clear();
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node_id);
                                s.draw_order.retain(|id| *id != node_id);
                                s.draw_order.push(node_id);
                            });
                        }
                    }
                    Outcome::Reject(sev, msg) => self.show_toast(cx.app, cx.window, sev, msg),
                    Outcome::Ignore => {}
                }
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
            _ => {}
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

            let is_active = menu.active_item == ix;
            let is_hovered = menu.hovered_item == Some(ix);
            if (is_hovered || is_active) && item.enabled {
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

    fn paint_toast<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        toast: &ToastState,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_h: f32,
    ) {
        let margin = 12.0 / zoom;
        let pad = 10.0 / zoom;
        let max_w = 420.0 / zoom;

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(max_w - 2.0 * pad)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        let (blob, metrics) =
            cx.services
                .text()
                .prepare(toast.message.as_ref(), &text_style, constraints);
        self.text_blobs.push(blob);

        let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(120.0 / zoom, max_w);
        let box_h = metrics.size.height.0 + 2.0 * pad;

        let x = viewport_origin_x + margin;
        let y = viewport_origin_y + viewport_h - box_h - margin;
        let rect = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(box_w), Px(box_h)));

        let border_color = match toast.severity {
            DiagnosticSeverity::Info => Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
            DiagnosticSeverity::Warning => Color {
                r: 0.95,
                g: 0.75,
                b: 0.20,
                a: 1.0,
            },
            DiagnosticSeverity::Error => Color {
                r: 0.90,
                g: 0.35,
                b: 0.35,
                a: 1.0,
            },
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(70),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0 / zoom)),
            border_color,
            corner_radii: Corners::all(Px(6.0 / zoom)),
        });

        let text_x = Px(rect.origin.x.0 + pad);
        let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(71),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.style.context_menu_text,
        });
    }

    fn paint_wire_drag_hint<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        wire_drag: &WireDrag,
        zoom: f32,
    ) {
        let text = match &wire_drag.kind {
            WireDragKind::New { bundle, .. } if bundle.len() > 1 => {
                Arc::<str>::from(format!("Bundle: {}", bundle.len()))
            }
            WireDragKind::ReconnectMany { edges } if edges.len() > 1 => {
                Arc::<str>::from(format!("Yank: {}", edges.len()))
            }
            _ => return,
        };

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let pad = 8.0 / zoom;
        let max_w = 220.0 / zoom;
        let constraints = TextConstraints {
            max_width: Some(Px(max_w - 2.0 * pad)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        let (blob, metrics) = cx
            .services
            .text()
            .prepare(text.as_ref(), &text_style, constraints);
        self.text_blobs.push(blob);

        let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(72.0 / zoom, max_w);
        let box_h = metrics.size.height.0 + 2.0 * pad;

        let offset_x = 14.0 / zoom;
        let offset_y = 12.0 / zoom;
        let rect = Rect::new(
            Point::new(
                Px(wire_drag.pos.x.0 + offset_x),
                Px(wire_drag.pos.y.0 + offset_y),
            ),
            Size::new(Px(box_w), Px(box_h)),
        );

        let border_color = if snapshot.interaction.connection_mode == NodeGraphConnectionMode::Loose
            && self.interaction.hover_port.is_some()
            && !self.interaction.hover_port_valid
        {
            if self.interaction.hover_port_convertible {
                Color {
                    r: 0.95,
                    g: 0.75,
                    b: 0.20,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.90,
                    g: 0.35,
                    b: 0.35,
                    a: 1.0,
                }
            }
        } else {
            self.style.context_menu_border
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(69),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0 / zoom)),
            border_color,
            corner_radii: Corners::all(Px(6.0 / zoom)),
        });

        let text_x = Px(rect.origin.x.0 + pad);
        let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(70),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.style.context_menu_text,
        });
    }

    fn yank_edges_from_port(graph: &Graph, port: PortId) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        let Some(p) = graph.ports.get(&port) else {
            return Vec::new();
        };

        let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();
        match p.dir {
            PortDirection::Out => {
                for (edge_id, edge) in &graph.edges {
                    if edge.from == port {
                        out.push((*edge_id, EdgeEndpoint::From, edge.to));
                    }
                }
            }
            PortDirection::In => {
                for (edge_id, edge) in &graph.edges {
                    if edge.to == port {
                        out.push((*edge_id, EdgeEndpoint::To, edge.from));
                    }
                }
            }
        }
        out
    }

    fn should_add_bundle_port(
        graph: &Graph,
        from: PortId,
        bundle: &[PortId],
        candidate: PortId,
    ) -> bool {
        if candidate == from || bundle.contains(&candidate) {
            return false;
        }
        let Some(from_port) = graph.ports.get(&from) else {
            return false;
        };
        let Some(candidate_port) = graph.ports.get(&candidate) else {
            return false;
        };
        candidate_port.dir == from_port.dir
    }

    fn pick_reconnect_endpoint(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        pos: Point,
        reconnect_radius_screen: f32,
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

        if reconnect_radius_screen.is_finite() && reconnect_radius_screen > 0.0 {
            let r = reconnect_radius_screen / zoom;
            let r2 = r * r;
            let min_d2 = d2_from.min(d2_to);
            if min_d2 > r2 {
                return None;
            }
        }

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
            Event::Timer { token } => {
                if self
                    .interaction
                    .toast
                    .as_ref()
                    .is_some_and(|t| t.timer == *token)
                {
                    self.interaction.toast = None;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::KeyDown { key, .. } => {
                if *key == fret_core::KeyCode::Escape {
                    if self.interaction.context_menu.take().is_some() {
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }

                    let mut canceled = false;
                    if self.interaction.wire_drag.take().is_some() {
                        canceled = true;
                    }
                    if self.interaction.edge_drag.take().is_some() {
                        canceled = true;
                    }
                    if self.interaction.node_drag.take().is_some() {
                        canceled = true;
                    }
                    if self.interaction.pending_node_drag.take().is_some() {
                        canceled = true;
                    }
                    if self.interaction.panning {
                        self.interaction.panning = false;
                        canceled = true;
                    }
                    self.interaction.hover_port = None;
                    self.interaction.hover_port_valid = false;
                    self.interaction.hover_port_convertible = false;
                    self.interaction.hover_edge = None;

                    if canceled {
                        cx.release_pointer_capture();
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    return;
                }

                if let Some(menu) = self.interaction.context_menu.as_mut() {
                    match *key {
                        fret_core::KeyCode::ArrowDown => {
                            let n = menu.items.len();
                            if n > 0 {
                                let mut ix = (menu.active_item + 1) % n;
                                for _ in 0..n {
                                    if menu.items.get(ix).is_some_and(|it| it.enabled) {
                                        break;
                                    }
                                    ix = (ix + 1) % n;
                                }
                                menu.active_item = ix;
                            }
                            menu.typeahead.clear();
                            cx.stop_propagation();
                            cx.request_redraw();
                            cx.invalidate_self(Invalidation::Paint);
                            return;
                        }
                        fret_core::KeyCode::ArrowUp => {
                            let n = menu.items.len();
                            if n > 0 {
                                let mut ix = if menu.active_item == 0 {
                                    n - 1
                                } else {
                                    menu.active_item - 1
                                };
                                for _ in 0..n {
                                    if menu.items.get(ix).is_some_and(|it| it.enabled) {
                                        break;
                                    }
                                    ix = if ix == 0 { n - 1 } else { ix - 1 };
                                }
                                menu.active_item = ix;
                            }
                            menu.typeahead.clear();
                            cx.stop_propagation();
                            cx.request_redraw();
                            cx.invalidate_self(Invalidation::Paint);
                            return;
                        }
                        fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
                            let ix = menu.active_item.min(menu.items.len().saturating_sub(1));
                            let item = menu.items.get(ix).cloned();
                            let target = menu.target.clone();
                            let invoked_at = menu.invoked_at;
                            let candidates = menu.candidates.clone();
                            self.interaction.context_menu = None;

                            if let Some(item) = item
                                && item.enabled
                            {
                                self.activate_context_menu_item(
                                    cx,
                                    &target,
                                    invoked_at,
                                    item,
                                    &candidates,
                                );
                            }

                            cx.stop_propagation();
                            cx.request_redraw();
                            cx.invalidate_self(Invalidation::Paint);
                            return;
                        }
                        fret_core::KeyCode::Backspace => {
                            if !menu.typeahead.is_empty() {
                                menu.typeahead.pop();
                                cx.stop_propagation();
                                cx.request_redraw();
                                cx.invalidate_self(Invalidation::Paint);
                                return;
                            }
                        }
                        _ => {}
                    }

                    if let Some(ch) = fret_core::keycode_to_ascii_lowercase(*key) {
                        let try_find = |needle: &str| -> Option<usize> {
                            if needle.is_empty() {
                                return None;
                            }
                            menu.items.iter().position(|it| {
                                it.enabled
                                    && it.label.as_ref().to_ascii_lowercase().starts_with(needle)
                            })
                        };

                        menu.typeahead.push(ch);
                        let mut needle = menu.typeahead.to_ascii_lowercase();
                        let mut hit = try_find(&needle);
                        if hit.is_none() {
                            needle.clear();
                            needle.push(ch);
                            hit = try_find(&needle);
                            if hit.is_some() {
                                menu.typeahead.clear();
                                menu.typeahead.push(ch);
                            }
                        }

                        if let Some(ix) = hit {
                            menu.active_item = ix.min(menu.items.len().saturating_sub(1));
                        }

                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
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

                if *button == MouseButton::Left {
                    if let Some(command) = self.close_command.clone() {
                        let rect = Self::close_button_rect(snapshot.pan, zoom);
                        if Self::rect_contains(rect, *position) {
                            cx.dispatch_command(command);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                if let Some(menu) = self.interaction.context_menu.as_mut() {
                    match button {
                        MouseButton::Left => {
                            if let Some(ix) =
                                hit_context_menu_item(&self.style, menu, *position, zoom)
                            {
                                let item = menu.items.get(ix).cloned();
                                let target = menu.target.clone();
                                let invoked_at = menu.invoked_at;
                                let candidates = menu.candidates.clone();
                                self.interaction.context_menu = None;
                                if let Some(item) = item
                                    && item.enabled
                                {
                                    self.activate_context_menu_item(
                                        cx,
                                        &target,
                                        invoked_at,
                                        item,
                                        &candidates,
                                    );
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
                    self.interaction.pending_node_drag = None;
                    self.interaction.node_drag = None;
                    self.interaction.wire_drag = None;
                    self.interaction.edge_drag = None;
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
                                    label: Arc::<str>::from("Insert Node..."),
                                    enabled: true,
                                    action: NodeGraphContextMenuAction::OpenInsertNodePicker,
                                });
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
                    let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
                    self.interaction.context_menu = Some(ContextMenuState {
                        origin,
                        invoked_at: *position,
                        target: ContextMenuTarget::Edge(edge),
                        items,
                        candidates: Vec::new(),
                        hovered_item: None,
                        active_item,
                        typeahead: String::new(),
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
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.edge_drag = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        let yank = (modifiers.ctrl || modifiers.meta).then(|| {
                            let this = &*self;
                            this.graph
                                .read_ref(cx.app, |graph| Self::yank_edges_from_port(graph, port))
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

                        self.interaction.wire_drag = Some(WireDrag {
                            kind,
                            pos: *position,
                        });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Node(node, rect) => {
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.wire_drag = None;
                        self.interaction.edge_drag = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        let offset = Point::new(
                            Px(position.x.0 - rect.origin.x.0),
                            Px(position.y.0 - rect.origin.y.0),
                        );
                        self.interaction.pending_node_drag = Some(PendingNodeDrag {
                            node,
                            grab_offset: offset,
                            start_pos: *position,
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
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.wire_drag = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
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
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.wire_drag = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
                            s.selected_edges.clear();
                        });
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position,
                modifiers,
                ..
            }) => {
                let Some(last) = self.interaction.last_pos else {
                    self.interaction.last_pos = Some(*position);
                    return;
                };
                let delta = Point::new(Px(position.x.0 - last.x.0), Px(position.y.0 - last.y.0));
                self.interaction.last_pos = Some(*position);

                if self.close_command.is_some()
                    && self.interaction.node_drag.is_none()
                    && self.interaction.wire_drag.is_none()
                    && self.interaction.edge_drag.is_none()
                    && !self.interaction.panning
                {
                    let rect = Self::close_button_rect(snapshot.pan, zoom);
                    if Self::rect_contains(rect, *position) {
                        cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
                    }
                }

                if self.interaction.panning {
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += delta.x.0;
                        s.pan.y += delta.y.0;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if self.interaction.node_drag.is_none() {
                    if let Some(pending) = self.interaction.pending_node_drag.clone() {
                        let threshold_screen = snapshot.interaction.node_drag_threshold.max(0.0);
                        let threshold_graph = threshold_screen / zoom;
                        let dx = position.x.0 - pending.start_pos.x.0;
                        let dy = position.y.0 - pending.start_pos.y.0;
                        if threshold_graph <= 0.0
                            || dx * dx + dy * dy >= threshold_graph * threshold_graph
                        {
                            self.interaction.pending_node_drag = None;
                            self.interaction.node_drag = Some(NodeDrag {
                                node: pending.node,
                                grab_offset: pending.grab_offset,
                            });
                        } else {
                            return;
                        }
                    }
                }

                if let Some(drag) = &self.interaction.node_drag {
                    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
                        .then(|| Self::auto_pan_delta(&snapshot, *position, cx.bounds))
                        .unwrap_or_default();
                    let new_pos = Point::new(
                        Px(position.x.0 - drag.grab_offset.x.0 - auto_pan_delta.x),
                        Px(position.y.0 - drag.grab_offset.y.0 - auto_pan_delta.y),
                    );
                    let id = drag.node;
                    let snap_to_grid = snapshot.interaction.snap_to_grid;
                    let snap_grid = snapshot.interaction.snap_grid;
                    let _ = self.graph.update(cx.app, |g, _cx| {
                        let Some(node) = g.nodes.get(&id) else {
                            return;
                        };
                        let from = node.pos;
                        let mut to = CanvasPoint {
                            x: new_pos.x.0,
                            y: new_pos.y.0,
                        };
                        if snap_to_grid {
                            to = Self::snap_canvas_point(to, snap_grid);
                        }
                        let tx = GraphTransaction {
                            label: None,
                            ops: vec![GraphOp::SetNodePos { id, from, to }],
                        };
                        let _ = apply_transaction(g, &tx);
                    });
                    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
                        self.update_view_state(cx.app, |s| {
                            s.pan.x += auto_pan_delta.x;
                            s.pan.y += auto_pan_delta.y;
                        });
                    }
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if let Some(mut w) = self.interaction.wire_drag.take() {
                    let auto_pan_delta = (snapshot.interaction.auto_pan.on_connect)
                        .then(|| Self::auto_pan_delta(&snapshot, *position, cx.bounds))
                        .unwrap_or_default();
                    w.pos = Point::new(
                        Px(position.x.0 - auto_pan_delta.x),
                        Px(position.y.0 - auto_pan_delta.y),
                    );
                    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
                        self.update_view_state(cx.app, |s| {
                            s.pan.x += auto_pan_delta.x;
                            s.pan.y += auto_pan_delta.y;
                        });
                    }

                    let pos = w.pos;

                    if modifiers.shift {
                        if let WireDragKind::New { from, bundle } = &mut w.kind {
                            let candidate = {
                                let this = &*self;
                                this.graph
                                    .read_ref(cx.app, |graph| this.hit_port(graph, pos, zoom))
                                    .ok()
                                    .flatten()
                            };

                            if let Some(candidate) = candidate {
                                let should_add = {
                                    let this = &*self;
                                    this.graph
                                        .read_ref(cx.app, |graph| {
                                            Self::should_add_bundle_port(
                                                graph, *from, bundle, candidate,
                                            )
                                        })
                                        .ok()
                                        .unwrap_or(false)
                                };
                                if should_add {
                                    bundle.push(candidate);
                                }
                            }
                        }
                    }

                    let from_port = match &w.kind {
                        WireDragKind::New { from, .. } => Some(*from),
                        WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                        WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
                    };

                    let new_hover = if let Some(from_port) = from_port {
                        let this = &*self;
                        this.graph
                            .read_ref(cx.app, |graph| {
                                this.pick_target_port(graph, &snapshot, from_port, pos, zoom)
                            })
                            .ok()
                            .flatten()
                    } else {
                        None
                    };

                    let new_hover_valid = if let Some(target) = new_hover {
                        let presenter = &mut *self.presenter;
                        self.graph
                            .read_ref(cx.app, |graph| {
                                let mut scratch = graph.clone();
                                match &w.kind {
                                    WireDragKind::New { from, bundle } => {
                                        let sources = if bundle.is_empty() {
                                            std::slice::from_ref(from)
                                        } else {
                                            bundle.as_slice()
                                        };
                                        let mut any_accept = false;
                                        for src in sources {
                                            let plan =
                                                presenter.plan_connect(&scratch, *src, target);
                                            if plan.decision != ConnectDecision::Accept {
                                                continue;
                                            }
                                            any_accept = true;
                                            let tx = GraphTransaction {
                                                label: None,
                                                ops: plan.ops.clone(),
                                            };
                                            let _ = apply_transaction(&mut scratch, &tx);
                                        }
                                        any_accept
                                    }
                                    WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
                                        presenter
                                            .plan_reconnect_edge(&scratch, *edge, *endpoint, target)
                                            .decision,
                                        ConnectDecision::Accept
                                    ),
                                    WireDragKind::ReconnectMany { edges } => {
                                        let mut any_accept = false;
                                        for (edge, endpoint, _fixed) in edges {
                                            let plan = presenter.plan_reconnect_edge(
                                                &scratch, *edge, *endpoint, target,
                                            );
                                            if plan.decision != ConnectDecision::Accept {
                                                continue;
                                            }
                                            any_accept = true;
                                            let tx = GraphTransaction {
                                                label: None,
                                                ops: plan.ops.clone(),
                                            };
                                            let _ = apply_transaction(&mut scratch, &tx);
                                        }
                                        any_accept
                                    }
                                }
                            })
                            .ok()
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let new_hover_convertible = if !new_hover_valid {
                        if let Some(target) = new_hover {
                            match &w.kind {
                                WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                                    let presenter = &mut *self.presenter;
                                    self.graph
                                        .read_ref(cx.app, |graph| {
                                            conversion::is_convertible(
                                                presenter, graph, *from, target,
                                            )
                                        })
                                        .ok()
                                        .unwrap_or(false)
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if self.interaction.hover_port != new_hover
                        || self.interaction.hover_port_valid != new_hover_valid
                        || self.interaction.hover_port_convertible != new_hover_convertible
                    {
                        self.interaction.hover_port = new_hover;
                        self.interaction.hover_port_valid = new_hover_valid;
                        self.interaction.hover_port_convertible = new_hover_convertible;
                    }

                    self.interaction.wire_drag = Some(w);
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
                                        snapshot.interaction.reconnect_radius,
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
                        if let Some(ix) = new_hover {
                            if menu.items.get(ix).is_some_and(|it| it.enabled) {
                                menu.active_item = ix.min(menu.items.len().saturating_sub(1));
                                menu.typeahead.clear();
                            }
                        }
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
                    if self.interaction.node_drag.is_some()
                        || self.interaction.pending_node_drag.is_some()
                    {
                        self.interaction.node_drag = None;
                        self.interaction.pending_node_drag = None;
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }

                    if let Some(w) = self.interaction.wire_drag.take() {
                        let from_port = match &w.kind {
                            WireDragKind::New { from, .. } => Some(*from),
                            WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                            WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
                        };
                        let target = from_port.and_then(|from_port| {
                            let this = &*self;
                            this.graph
                                .read_ref(cx.app, |graph| {
                                    this.pick_target_port(graph, &snapshot, from_port, w.pos, zoom)
                                })
                                .ok()
                                .flatten()
                        });
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;

                        match w.kind {
                            WireDragKind::New { from, bundle } => {
                                if let Some(target) = target {
                                    enum Outcome {
                                        Apply(Vec<GraphOp>),
                                        Reject(DiagnosticSeverity, Arc<str>),
                                        Ignore,
                                        OpenConversionPicker(Vec<InsertNodeCandidate>),
                                    }

                                    let (outcome, toast) = {
                                        let presenter = &mut *self.presenter;
                                        let style = self.style.clone();
                                        self.graph
                                            .read_ref(cx.app, |graph| {
                                                let mut scratch = graph.clone();
                                                let sources: Vec<PortId> = if bundle.is_empty() {
                                                    vec![from]
                                                } else {
                                                    bundle
                                                };
                                                let allow_convert = sources.len() == 1;
                                                let convert_at = CanvasPoint {
                                                    x: w.pos.x.0,
                                                    y: w.pos.y.0,
                                                };
                                                let mut picker: Option<Vec<InsertNodeCandidate>> = None;
                                                let mut ops_all: Vec<GraphOp> = Vec::new();
                                                let mut toast: Option<(
                                                    DiagnosticSeverity,
                                                    Arc<str>,
                                                )> = None;

                                                for src in sources {
                                                    let plan = presenter
                                                        .plan_connect(&scratch, src, target);
                                                    match plan.decision {
                                                        ConnectDecision::Accept => {
                                                            let tx = GraphTransaction {
                                                                label: None,
                                                                ops: plan.ops.clone(),
                                                            };
                                                            let _ = apply_transaction(
                                                                &mut scratch,
                                                                &tx,
                                                            );
                                                            ops_all.extend(plan.ops);
                                                        }
                                                        ConnectDecision::Reject => {
                                                            if allow_convert {
                                                                let conversions = presenter
                                                                    .list_conversions(
                                                                        &scratch, src, target,
                                                                    );
                                                                if conversions.len() > 1 {
                                                                    picker = Some(
                                                                        conversion::build_picker_candidates(
                                                                            presenter,
                                                                            &scratch,
                                                                            src,
                                                                            target,
                                                                            conversions,
                                                                        ),
                                                                    );
                                                                    break;
                                                                }
                                                                if conversions.len() == 1 {
                                                                    if let Some(insert_plan) =
                                                                        conversion::try_auto_insert_conversion(
                                                                            presenter,
                                                                            &scratch,
                                                                            &style,
                                                                            zoom,
                                                                            src,
                                                                            target,
                                                                            convert_at,
                                                                            &conversions,
                                                                        )
                                                                    {
                                                                        if insert_plan.decision
                                                                            == ConnectDecision::Accept
                                                                        {
                                                                            let tx = GraphTransaction {
                                                                                label: None,
                                                                                ops: insert_plan.ops.clone(),
                                                                            };
                                                                            let _ = apply_transaction(
                                                                                &mut scratch,
                                                                                &tx,
                                                                            );
                                                                            ops_all.extend(insert_plan.ops);
                                                                            continue;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            if toast.is_none() {
                                                                toast =
                                                                    Self::toast_from_diagnostics(
                                                                        &plan.diagnostics,
                                                                    );
                                                            }
                                                        }
                                                    }
                                                }

                                                let outcome = if let Some(picker) = picker {
                                                    Outcome::OpenConversionPicker(picker)
                                                } else if ops_all.is_empty() {
                                                    if let Some((sev, msg)) = toast.clone() {
                                                        Outcome::Reject(sev, msg)
                                                    } else {
                                                        Outcome::Ignore
                                                    }
                                                } else {
                                                    Outcome::Apply(ops_all)
                                                };
                                                (outcome, toast)
                                            })
                                            .ok()
                                            .unwrap_or((Outcome::Ignore, None))
                                    };

                                    match outcome {
                                        Outcome::Apply(ops) => {
                                            self.apply_ops(cx.app, ops);
                                            if let Some((sev, msg)) = toast {
                                                self.show_toast(cx.app, cx.window, sev, msg);
                                            }
                                        }
                                        Outcome::OpenConversionPicker(candidates) => {
                                            let mut items: Vec<NodeGraphContextMenuItem> =
                                                Vec::new();
                                            for (ix, c) in candidates.iter().enumerate() {
                                                items.push(NodeGraphContextMenuItem {
                                                    label: c.label.clone(),
                                                    enabled: c.enabled,
                                                    action: NodeGraphContextMenuAction::InsertNodeCandidate(ix),
                                                });
                                            }

                                            let origin = self.clamp_context_menu_origin(
                                                *position,
                                                items.len(),
                                                cx.bounds,
                                                &snapshot,
                                            );
                                            let active_item =
                                                items.iter().position(|it| it.enabled).unwrap_or(0);
                                            self.interaction.context_menu =
                                                Some(ContextMenuState {
                                                    origin,
                                                    invoked_at: *position,
                                                    target:
                                                        ContextMenuTarget::ConnectionConvertPicker {
                                                            from,
                                                            to: target,
                                                            at: CanvasPoint {
                                                                x: w.pos.x.0,
                                                                y: w.pos.y.0,
                                                            },
                                                        },
                                                    items,
                                                    candidates,
                                                    hovered_item: None,
                                                    active_item,
                                                    typeahead: String::new(),
                                                });
                                        }
                                        Outcome::Reject(sev, msg) => {
                                            self.show_toast(cx.app, cx.window, sev, msg);
                                        }
                                        Outcome::Ignore => {}
                                    }
                                }
                            }
                            WireDragKind::Reconnect {
                                edge,
                                endpoint,
                                fixed: _fixed,
                            } => {
                                if let Some(target) = target {
                                    enum Outcome {
                                        Apply(Vec<GraphOp>),
                                        Reject(DiagnosticSeverity, Arc<str>),
                                        Ignore,
                                    }

                                    let outcome = {
                                        let presenter = &mut *self.presenter;
                                        self.graph
                                            .read_ref(cx.app, |graph| {
                                                let plan = presenter.plan_reconnect_edge(
                                                    graph, edge, endpoint, target,
                                                );
                                                match plan.decision {
                                                    ConnectDecision::Accept => {
                                                        Outcome::Apply(plan.ops)
                                                    }
                                                    ConnectDecision::Reject => {
                                                        Self::toast_from_diagnostics(
                                                            &plan.diagnostics,
                                                        )
                                                        .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                                        .unwrap_or(Outcome::Ignore)
                                                    }
                                                }
                                            })
                                            .ok()
                                            .unwrap_or(Outcome::Ignore)
                                    };
                                    match outcome {
                                        Outcome::Apply(ops) => self.apply_ops(cx.app, ops),
                                        Outcome::Reject(sev, msg) => {
                                            self.show_toast(cx.app, cx.window, sev, msg);
                                        }
                                        Outcome::Ignore => {}
                                    }
                                }
                            }
                            WireDragKind::ReconnectMany { edges } => {
                                if let Some(target) = target {
                                    let presenter = &mut *self.presenter;
                                    let (ops_all, toast) = self
                                        .graph
                                        .read_ref(cx.app, |graph| {
                                            let mut scratch = graph.clone();
                                            let mut ops_all: Vec<GraphOp> = Vec::new();
                                            let mut toast: Option<(DiagnosticSeverity, Arc<str>)> =
                                                None;

                                            for (edge, endpoint, _fixed) in edges {
                                                let plan = presenter.plan_reconnect_edge(
                                                    &scratch, edge, endpoint, target,
                                                );
                                                match plan.decision {
                                                    ConnectDecision::Accept => {
                                                        let tx = GraphTransaction {
                                                            label: None,
                                                            ops: plan.ops.clone(),
                                                        };
                                                        let _ =
                                                            apply_transaction(&mut scratch, &tx);
                                                        ops_all.extend(plan.ops);
                                                    }
                                                    ConnectDecision::Reject => {
                                                        if toast.is_none() {
                                                            toast = Self::toast_from_diagnostics(
                                                                &plan.diagnostics,
                                                            );
                                                        }
                                                    }
                                                }
                                            }

                                            (ops_all, toast)
                                        })
                                        .ok()
                                        .unwrap_or_default();

                                    if !ops_all.is_empty() {
                                        self.apply_ops(cx.app, ops_all);
                                    }
                                    if let Some((sev, msg)) = toast {
                                        self.show_toast(cx.app, cx.window, sev, msg);
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
            nodes: Vec<(GraphNodeId, Rect, bool, Arc<str>)>,
            pins: Vec<(PortId, Rect, Color)>,
            port_labels: HashMap<PortId, (Arc<str>, PortDirection)>,
            port_centers: HashMap<PortId, Point>,
        }

        let hovered_edge = self.interaction.hover_edge;
        let hovered_port = self.interaction.hover_port;
        let hovered_port_valid = self.interaction.hover_port_valid;
        let hovered_port_convertible = self.interaction.hover_port_convertible;
        let wire_drag = self.interaction.wire_drag.clone();
        let marked_ports: HashSet<PortId> = match wire_drag.as_ref().map(|w| &w.kind) {
            Some(WireDragKind::New { bundle, .. }) if bundle.len() > 1 => {
                bundle.iter().copied().collect()
            }
            Some(WireDragKind::ReconnectMany { edges }) if edges.len() > 1 => edges
                .iter()
                .map(|(_edge, _endpoint, fixed)| *fixed)
                .collect(),
            _ => HashSet::new(),
        };

        let render = {
            let selected: HashSet<GraphNodeId> = snapshot.selected_nodes.iter().copied().collect();
            let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();
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
                        let title = presenter.node_title(graph, *node);
                        out.nodes.push((*node, rect, is_selected, title));

                        let (inputs, outputs) = this.node_ports(graph, *node);
                        for port in inputs.iter().chain(outputs.iter()) {
                            if let Some(center) = this.port_center(graph, *node, *port, zoom) {
                                out.port_centers.insert(*port, center);
                                if let Some(p) = graph.ports.get(port) {
                                    out.port_labels
                                        .insert(*port, (presenter.port_label(graph, *port), p.dir));
                                }
                                let color = presenter.port_color(graph, *port, &this.style);
                                let pin_r = this.style.pin_radius / zoom;
                                let rect = Rect::new(
                                    Point::new(Px(center.x.0 - pin_r), Px(center.y.0 - pin_r)),
                                    Size::new(Px(2.0 * pin_r), Px(2.0 * pin_r)),
                                );
                                out.pins.push((*port, rect, color));
                            }
                        }
                    }

                    for (&edge_id, edge) in &graph.edges {
                        if this
                            .interaction
                            .wire_drag
                            .as_ref()
                            .is_some_and(|w| Self::wire_drag_suppresses_edge(&w.kind, edge_id))
                        {
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
            let to = hovered_port
                .filter(|_| hovered_port_valid || hovered_port_convertible)
                .and_then(|port| render.port_centers.get(&port).copied())
                .unwrap_or(w.pos);
            let color =
                if hovered_port.is_some() && !hovered_port_valid && !hovered_port_convertible {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 0.95,
                    }
                } else if hovered_port.is_some() && hovered_port_convertible && !hovered_port_valid
                {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 0.95,
                    }
                } else {
                    self.style.wire_color_preview
                };

            let mut draw_preview = |from: Point| {
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
            };

            match &w.kind {
                WireDragKind::New { from, bundle } => {
                    let ports = if bundle.is_empty() {
                        std::slice::from_ref(from)
                    } else {
                        bundle.as_slice()
                    };
                    for port in ports {
                        if let Some(from) = render.port_centers.get(port).copied() {
                            draw_preview(from);
                        }
                    }
                }
                WireDragKind::Reconnect { fixed, .. } => {
                    if let Some(from) = render.port_centers.get(fixed).copied() {
                        draw_preview(from);
                    }
                }
                WireDragKind::ReconnectMany { edges } => {
                    for (_edge, _endpoint, fixed) in edges {
                        if let Some(from) = render.port_centers.get(fixed).copied() {
                            draw_preview(from);
                        }
                    }
                }
            }
        }

        let mut node_text_style = self.style.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(lh) = node_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let corner = Px(8.0 / zoom);
        let title_pad = self.style.node_padding / zoom;
        let title_h = self.style.node_header_height / zoom;

        for (_node, rect, is_selected, title) in &render.nodes {
            let rect = *rect;
            let border_color = if *is_selected {
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

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare(title.as_ref(), &node_text_style, constraints);
                self.text_blobs.push(blob);

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
                let text_y = Px(inner_y + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }
        }

        let pin_r = self.style.pin_radius / zoom;
        let pin_gap = 8.0 / zoom;
        let label_max_w = (self.style.node_width
            - 2.0 * self.style.node_padding
            - 2.0 * (self.style.pin_radius + 8.0))
            .max(0.0)
            / zoom;
        let port_constraints = TextConstraints {
            max_width: Some(Px(label_max_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        for (port_id, (label, dir)) in &render.port_labels {
            let Some(center) = render.port_centers.get(port_id).copied() else {
                continue;
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(label.as_ref(), &node_text_style, port_constraints);
            self.text_blobs.push(blob);

            let y = Px(center.y.0 - 0.5 * metrics.size.height.0 + metrics.baseline.0);
            let x = match dir {
                PortDirection::In => Px(center.x.0 + pin_r + pin_gap),
                PortDirection::Out => Px(center.x.0 - pin_r - pin_gap - metrics.size.width.0),
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(4),
                origin: Point::new(x, y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        for (port_id, rect, color) in render.pins {
            if marked_ports.contains(&port_id) {
                let pad = 5.0 / zoom;
                let marker_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * marker_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: marker_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(1.0 / zoom)),
                    border_color: Color {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: 0.55,
                    },
                    corner_radii: Corners::all(r),
                });
            }

            if hovered_port == Some(port_id) {
                let border_color = if hovered_port_valid {
                    color
                } else if hovered_port_convertible {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 1.0,
                    }
                };
                let pad = 2.0 / zoom;
                let hover_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * hover_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: hover_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(2.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(r),
                });
            }

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

        if self.close_command.is_some() {
            let rect = Self::close_button_rect(snapshot.pan, zoom);
            let hovered = self
                .interaction
                .last_pos
                .is_some_and(|p| Self::rect_contains(rect, p));

            let background = if hovered {
                self.style.context_menu_hover_background
            } else {
                self.style.context_menu_background
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(60),
                rect,
                background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color: self.style.context_menu_border,
                corner_radii: Corners::all(Px(6.0 / zoom)),
            });

            let mut text_style = self.style.context_menu_text_style.clone();
            text_style.size = Px(text_style.size.0 / zoom);
            if let Some(lh) = text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }
            let pad = 10.0 / zoom;
            let constraints = TextConstraints {
                max_width: Some(Px((rect.size.width.0 - 2.0 * pad).max(0.0))),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor * zoom,
            };
            let (blob, metrics) = cx
                .services
                .text()
                .prepare("Close", &text_style, constraints);
            self.text_blobs.push(blob);

            let text_x = Px(rect.origin.x.0 + pad);
            let inner_y = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(61),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        if let Some(wire_drag) = wire_drag {
            self.paint_wire_drag_hint(cx, &snapshot, &wire_drag, zoom);
        }

        if let Some(menu) = self.interaction.context_menu.clone() {
            self.paint_context_menu(cx, &menu, zoom);
        }

        if let Some(toast) = self.interaction.toast.clone() {
            self.paint_toast(
                cx,
                &toast,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_h,
            );
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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::core::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use crate::rules::EdgeEndpoint;

    use super::NodeGraphCanvas;

    #[test]
    fn yank_edges_from_port_returns_all_incident_edges() {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let n1 = NodeId::new();
        let p_out = PortId::new();
        graph.nodes.insert(
            n1,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                collapsed: false,
                ports: vec![p_out],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_out,
            Port {
                node: n1,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: None,
                data: Value::Null,
            },
        );

        let n2 = NodeId::new();
        let p_in1 = PortId::new();
        let p_in2 = PortId::new();
        graph.nodes.insert(
            n2,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                collapsed: false,
                ports: vec![p_in1, p_in2],
                data: Value::Null,
            },
        );
        for (id, key) in [(p_in1, "in1"), (p_in2, "in2")] {
            graph.ports.insert(
                id,
                Port {
                    node: n2,
                    key: PortKey::new(key),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: None,
                    data: Value::Null,
                },
            );
        }

        let e1 = EdgeId::new();
        let e2 = EdgeId::new();
        graph.edges.insert(
            e1,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in1,
            },
        );
        graph.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in2,
            },
        );

        let from_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_out);
        assert_eq!(from_edges.len(), 2);
        assert!(from_edges.contains(&(e1, EdgeEndpoint::From, p_in1)));
        assert!(from_edges.contains(&(e2, EdgeEndpoint::From, p_in2)));

        let to_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_in1);
        assert_eq!(to_edges, vec![(e1, EdgeEndpoint::To, p_out)]);
    }

    #[test]
    fn should_add_bundle_port_requires_same_side_and_dedupes() {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");
        let n1 = NodeId::new();

        let p_out1 = PortId::new();
        let p_out2 = PortId::new();
        let p_in = PortId::new();

        graph.nodes.insert(
            n1,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                collapsed: false,
                ports: vec![p_out1, p_out2, p_in],
                data: Value::Null,
            },
        );

        graph.ports.insert(
            p_out1,
            Port {
                node: n1,
                key: PortKey::new("out1"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_out2,
            Port {
                node: n1,
                key: PortKey::new("out2"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_in,
            Port {
                node: n1,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: Value::Null,
            },
        );

        assert!(NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[p_out1],
            p_out2
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[p_out2],
            p_out2
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[],
            p_out1
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[],
            p_in
        ));
    }
}
