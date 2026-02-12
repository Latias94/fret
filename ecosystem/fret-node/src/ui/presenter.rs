use fret_core::Color;
use serde_json::Value;
use std::sync::Arc;

use crate::REROUTE_KIND;
use crate::core::{
    CanvasPoint, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};
use crate::interaction::NodeGraphConnectionMode;
#[cfg(feature = "kit")]
use crate::kit::profiles::DataflowProfile;
use crate::ops::GraphOp;
use crate::profile::GraphProfile;
use crate::rules::{
    ConnectPlan, EdgeEndpoint, InsertNodeSpec, InsertNodeTemplate, plan_connect_with_mode,
    plan_reconnect_edge_with_mode, plan_split_edge_by_inserting_node,
};
use crate::schema::NodeRegistry;
use crate::types::TypeDesc;
use fret_runtime::CommandId;

use super::canvas::NodeResizeHandle;
use super::style::NodeGraphStyle;
use fret_core::{Point, Rect};

/// Context menu actions surfaced by the canvas widget.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeGraphContextMenuAction {
    OpenInsertNodePicker,
    InsertNodeCandidate(usize),
    InsertReroute,
    DeleteEdge,
    Command(CommandId),
    Custom(u64),
}

/// A context menu item.
#[derive(Debug, Clone)]
pub struct NodeGraphContextMenuItem {
    pub label: Arc<str>,
    pub enabled: bool,
    pub action: NodeGraphContextMenuAction,
}

/// A candidate node kind for insertion.
#[derive(Debug, Clone)]
pub struct InsertNodeCandidate {
    pub kind: NodeKindKey,
    pub label: Arc<str>,
    pub enabled: bool,
    pub template: Option<InsertNodeTemplate>,
    pub payload: Value,
}

/// A presenter-provided port anchor hint, in node-local screen-space (logical px).
///
/// The canvas transform scales graph space by `zoom`. The node graph uses "semantic zoom" where
/// most UI elements remain readable at extreme zoom levels by keeping their screen-space sizes
/// stable. As a result:
/// - anchor inputs are expressed in screen-space pixels (logical px),
/// - the canvas converts them into graph/canvas space by dividing by `zoom` and offsetting by the
///   node's graph-space origin.
#[derive(Debug, Clone, Copy)]
pub struct PortAnchorHint {
    pub center: Point,
    pub bounds: Rect,
}

/// A bitset defining which resize handles are enabled for a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeResizeHandleSet {
    bits: u16,
}

impl NodeResizeHandleSet {
    const fn mask(handle: NodeResizeHandle) -> u16 {
        match handle {
            NodeResizeHandle::TopLeft => 1 << 0,
            NodeResizeHandle::Top => 1 << 1,
            NodeResizeHandle::TopRight => 1 << 2,
            NodeResizeHandle::Right => 1 << 3,
            NodeResizeHandle::BottomRight => 1 << 4,
            NodeResizeHandle::Bottom => 1 << 5,
            NodeResizeHandle::BottomLeft => 1 << 6,
            NodeResizeHandle::Left => 1 << 7,
        }
    }

    pub const NONE: Self = Self { bits: 0 };
    pub const ALL: Self = Self { bits: (1 << 8) - 1 };

    pub const fn none() -> Self {
        Self::NONE
    }

    pub const fn all() -> Self {
        Self::ALL
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    pub const fn bits(self) -> u16 {
        self.bits
    }

    pub const fn contains(self, handle: NodeResizeHandle) -> bool {
        (self.bits & Self::mask(handle)) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn insert(&mut self, handle: NodeResizeHandle) {
        self.bits |= Self::mask(handle);
    }

    pub fn remove(&mut self, handle: NodeResizeHandle) {
        self.bits &= !Self::mask(handle);
    }
}

impl Default for NodeResizeHandleSet {
    fn default() -> Self {
        Self::all()
    }
}

/// Optional per-node resize constraints expressed in screen-space pixels (logical px).
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeResizeConstraintsPx {
    /// Minimum node size override (width, height) in logical px.
    pub min_size_px: Option<(f32, f32)>,
    /// Maximum node size override (width, height) in logical px.
    pub max_size_px: Option<(f32, f32)>,
}

impl NodeResizeConstraintsPx {
    pub fn normalized(mut self) -> Self {
        let normalize = |v: &mut Option<(f32, f32)>| {
            if let Some((w, h)) = *v {
                if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
                    *v = None;
                }
            }
        };
        normalize(&mut self.min_size_px);
        normalize(&mut self.max_size_px);

        if let (Some(min), Some(max)) = (self.min_size_px, self.max_size_px) {
            self.max_size_px = Some((max.0.max(min.0), max.1.max(min.1)));
        }

        self
    }
}

/// Edge routing kind for the canvas renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeRouteKind {
    /// Smooth cubic Bezier.
    Bezier,
    /// Straight line.
    Straight,
    /// Orthogonal "step" routing with right-angle segments.
    Step,
}

impl Default for EdgeRouteKind {
    fn default() -> Self {
        Self::Bezier
    }
}

/// Marker kind for edge endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeMarkerKind {
    /// Filled triangle arrow head.
    Arrow,
}

/// Optional marker rendered at an edge endpoint.
#[derive(Debug, Clone)]
pub struct EdgeMarker {
    pub kind: EdgeMarkerKind,
    /// Marker size in screen-space pixels (logical px).
    pub size: f32,
}

impl EdgeMarker {
    pub fn arrow(size: f32) -> Self {
        Self {
            kind: EdgeMarkerKind::Arrow,
            size,
        }
    }
}

/// Optional per-edge rendering hints.
#[derive(Debug, Clone, Default)]
pub struct EdgeRenderHint {
    /// Optional label rendered near the edge center.
    pub label: Option<Arc<str>>,
    /// Optional color override (falls back to `edge_color(...)`).
    pub color: Option<Color>,
    /// Width multiplier applied before selection/hover multipliers.
    pub width_mul: f32,
    /// Edge routing kind.
    pub route: EdgeRouteKind,
    /// Optional marker rendered at the edge start.
    pub start_marker: Option<EdgeMarker>,
    /// Optional marker rendered at the edge end.
    pub end_marker: Option<EdgeMarker>,
}

impl EdgeRenderHint {
    pub fn normalized(mut self) -> Self {
        if !self.width_mul.is_finite() || self.width_mul <= 0.0 {
            self.width_mul = 1.0;
        }
        if let Some(m) = self.start_marker.as_mut() {
            if !m.size.is_finite() || m.size <= 0.0 {
                self.start_marker = None;
            }
        }
        if let Some(m) = self.end_marker.as_mut() {
            if !m.size.is_finite() || m.size <= 0.0 {
                self.end_marker = None;
            }
        }
        self
    }
}

/// Viewer/presenter surface for the node graph UI.
///
/// This is the primary extensibility point: domain code can define titles, styles, and connection
/// behavior without forking the editor widget.
pub trait NodeGraphPresenter {
    /// Revision that invalidates derived geometry caches.
    ///
    /// Implementations that provide dynamic geometry hints (e.g. measured sizes from a UI subtree)
    /// should bump this when the underlying measurements change.
    fn geometry_revision(&self) -> u64 {
        0
    }

    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str>;
    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str>;

    /// Accessible label for the node graph canvas root.
    ///
    /// This is used by `NodeGraphCanvas::semantics` as a baseline for assistive technologies.
    fn a11y_canvas_label(&self) -> Arc<str> {
        Arc::<str>::from("Node Graph Canvas")
    }

    /// Accessible label for a node.
    ///
    /// Defaults to `node_title(...)`.
    fn a11y_node_label(&self, graph: &Graph, node: NodeId) -> Option<Arc<str>> {
        Some(self.node_title(graph, node))
    }

    /// Accessible label for a port.
    ///
    /// Defaults to `port_label(...)`.
    fn a11y_port_label(&self, graph: &Graph, port: PortId) -> Option<Arc<str>> {
        Some(self.port_label(graph, port))
    }

    /// Accessible label for an edge.
    ///
    /// Defaults to `edge_render_hint(...).label` (when present).
    fn a11y_edge_label(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
    ) -> Option<Arc<str>> {
        self.edge_render_hint(graph, edge, style).label
    }

    /// Optional node body label.
    ///
    /// This is a low-friction extensibility point for MVP editor UIs that want to show simple,
    /// domain-derived content (e.g. constant values) without embedding an entire UI subtree.
    ///
    /// More advanced node content should eventually be modeled as a dedicated "node view" layer
    /// (see ADR 0126), but this hook is useful for early demos and diagnostics.
    fn node_body_label(&self, _graph: &Graph, _node: NodeId) -> Option<Arc<str>> {
        None
    }

    fn port_color(&self, graph: &Graph, port: PortId, style: &NodeGraphStyle) -> Color {
        let Some(p) = graph.ports.get(&port) else {
            return style.node_border;
        };
        match p.kind {
            PortKind::Data => style.pin_color_data,
            PortKind::Exec => style.pin_color_exec,
        }
    }

    fn edge_color(&self, graph: &Graph, edge: EdgeId, style: &NodeGraphStyle) -> Color {
        let Some(e) = graph.edges.get(&edge) else {
            return style.node_border;
        };
        match e.kind {
            crate::core::EdgeKind::Data => style.wire_color_data,
            crate::core::EdgeKind::Exec => style.wire_color_exec,
        }
    }

    /// Optional edge rendering hints (label/style/routing).
    fn edge_render_hint(
        &self,
        _graph: &Graph,
        _edge: EdgeId,
        _style: &NodeGraphStyle,
    ) -> EdgeRenderHint {
        EdgeRenderHint {
            width_mul: 1.0,
            ..EdgeRenderHint::default()
        }
    }

    /// Optional per-node size hint in screen-space pixels (logical px).
    ///
    /// When absent, the canvas derives size from `NodeGraphStyle` and port counts.
    fn node_size_hint_px(
        &mut self,
        _graph: &Graph,
        _node: NodeId,
        _style: &NodeGraphStyle,
    ) -> Option<(f32, f32)> {
        None
    }

    /// Optional per-port anchor hint in node-local screen-space (logical px).
    ///
    /// When absent, the canvas derives anchor positions from `NodeGraphStyle` and port ordering.
    fn port_anchor_hint(
        &mut self,
        _graph: &Graph,
        _node: NodeId,
        _port: PortId,
        _style: &NodeGraphStyle,
    ) -> Option<PortAnchorHint> {
        None
    }

    /// Enabled resize handles for a node.
    ///
    /// Returning `NodeResizeHandleSet::none()` disables resizing for the node.
    fn node_resize_handles(
        &self,
        _graph: &Graph,
        _node: NodeId,
        _style: &NodeGraphStyle,
    ) -> NodeResizeHandleSet {
        NodeResizeHandleSet::all()
    }

    /// Optional per-node resize constraints in screen-space pixels (logical px).
    ///
    /// These are combined with style- and port-driven minimum size, and with `node_extent` / group
    /// bounds used as maximum bounds. Returning `None` values keeps the defaults.
    fn node_resize_constraints_px(
        &self,
        _graph: &Graph,
        _node: NodeId,
        _style: &NodeGraphStyle,
    ) -> NodeResizeConstraintsPx {
        NodeResizeConstraintsPx::default()
    }

    /// Lists nodes that can be inserted into the graph from a palette (background insert).
    ///
    /// Returning an empty list disables the insert-node picker by default.
    fn list_insertable_nodes(&mut self, _graph: &Graph) -> Vec<InsertNodeCandidate> {
        Vec::new()
    }

    /// Lists nodes that can be inserted as part of a connection workflow.
    ///
    /// This is used when a user drops a connection onto empty space and the editor opens a node
    /// picker. Default behavior reuses `list_insertable_nodes`.
    fn list_insertable_nodes_for_connection(
        &mut self,
        graph: &Graph,
        from: PortId,
    ) -> Vec<InsertNodeCandidate> {
        let _ = from;
        self.list_insertable_nodes(graph)
    }

    /// Plans inserting a node into the graph (background insert).
    ///
    /// The returned ops must be valid reversible graph edits.
    fn plan_create_node(
        &mut self,
        _graph: &Graph,
        _candidate: &InsertNodeCandidate,
        _at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        Err(Arc::<str>::from("node insertion is not supported"))
    }

    /// Lists insertable nodes for split-edge workflows.
    ///
    /// Implementations may inspect the edge type and port types to return compatible candidates.
    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        let _ = (graph, edge);
        Vec::new()
    }

    /// Plans splitting an edge by inserting a node.
    ///
    /// Returning a rejected plan will surface diagnostics to the user.
    fn plan_split_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        node_kind: &NodeKindKey,
        at: CanvasPoint,
    ) -> ConnectPlan {
        if node_kind.0 != REROUTE_KIND {
            let _ = (graph, edge, node_kind, at);
            return ConnectPlan::reject("split-edge insertion is not supported");
        }

        let edge_id = edge;

        let Some(edge) = graph.edges.get(&edge_id) else {
            return ConnectPlan::reject("missing edge");
        };
        let Some(from_port) = graph.ports.get(&edge.from) else {
            return ConnectPlan::reject("missing edge.from port");
        };
        let Some(to_port) = graph.ports.get(&edge.to) else {
            return ConnectPlan::reject("missing edge.to port");
        };

        let port_kind = match edge.kind {
            EdgeKind::Data => PortKind::Data,
            EdgeKind::Exec => PortKind::Exec,
        };
        let ty = from_port.ty.clone().or_else(|| to_port.ty.clone());

        let node_id = NodeId::new();
        let in_port_id = PortId::new();
        let out_port_id = PortId::new();

        let node = Node {
            kind: node_kind.clone(),
            kind_version: 1,
            pos: at,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: ty.clone(),
            data: Value::default(),
        };

        let out_port = Port {
            node: node_id,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: port_kind,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty,
            data: Value::default(),
        };

        plan_split_edge_by_inserting_node(
            graph,
            edge_id,
            EdgeId::new(),
            InsertNodeSpec {
                node_id,
                node,
                ports: vec![(in_port_id, in_port), (out_port_id, out_port)],
                input: in_port_id,
                output: out_port_id,
            },
        )
    }

    /// Plans splitting an edge by inserting a selected candidate node.
    ///
    /// This allows candidate-specific configuration via `InsertNodeCandidate::payload` without
    /// forcing every option to be represented as a distinct `NodeKindKey`.
    fn plan_split_edge_candidate(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> ConnectPlan {
        if let Some(template) = &candidate.template {
            match template.instantiate(at) {
                Ok(spec) => plan_split_edge_by_inserting_node(graph, edge, EdgeId::new(), spec),
                Err(err) => ConnectPlan::reject(err),
            }
        } else {
            self.plan_split_edge(graph, edge, &candidate.kind, at)
        }
    }

    /// Fills the right-click context menu for an edge.
    ///
    /// The canvas will append built-in actions (e.g. `Delete`) after these items.
    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        _style: &NodeGraphStyle,
        _out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        let _ = (graph, edge);
    }

    /// Handles a custom context menu action.
    ///
    /// Returning `Some(ops)` applies them as a single transaction.
    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<GraphOp>> {
        let _ = (graph, edge, action);
        None
    }

    /// Connection decision point.
    ///
    /// Implementations may return a `ConnectPlan` that:
    /// - rejects the connection (diagnostics only),
    /// - accepts it with direct edge changes,
    /// - accepts it with additional ops (e.g. insert conversion nodes).
    fn plan_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        mode: NodeGraphConnectionMode,
    ) -> ConnectPlan {
        if let Some(profile) = self.profile_mut() {
            profile.plan_connect(graph, a, b, mode)
        } else {
            plan_connect_with_mode(graph, a, b, mode)
        }
    }

    /// Reconnection decision point (preserve edge identity when possible).
    fn plan_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
        mode: NodeGraphConnectionMode,
    ) -> ConnectPlan {
        plan_reconnect_edge_with_mode(graph, edge, endpoint, new_port, mode)
    }

    /// Optional profile hook for typed graphs and edit pipelines.
    ///
    /// Returning a profile enables:
    /// - typed `plan_connect` by default,
    /// - profile-driven concretization/validation when applying transactions.
    fn profile_mut(&mut self) -> Option<&mut (dyn GraphProfile + 'static)> {
        None
    }

    /// Returns the (possibly domain-derived) type of a port.
    ///
    /// This is a "typed graph profile" hook; the default implementation falls back to `Port::ty`.
    fn type_of_port(&self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        graph.ports.get(&port).and_then(|p| p.ty.clone())
    }

    /// Fast (UI-friendly) connectivity check for previews and hover states.
    ///
    /// Default implementation delegates to `plan_connect` but strips ops.
    fn can_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        mode: NodeGraphConnectionMode,
    ) -> ConnectPlan {
        let mut plan = self.plan_connect(graph, a, b, mode);
        plan.ops.clear();
        plan
    }

    /// Fast (UI-friendly) reconnect check for previews and hover states.
    ///
    /// Default implementation delegates to `plan_reconnect_edge` but strips ops.
    fn can_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
        mode: NodeGraphConnectionMode,
    ) -> ConnectPlan {
        let mut plan = self.plan_reconnect_edge(graph, edge, endpoint, new_port, mode);
        plan.ops.clear();
        plan
    }

    /// Lists conversion templates that could make a rejected connection possible.
    ///
    /// UI may use this to show "convertible" targets, and to optionally auto-insert a conversion
    /// node when there is exactly one unambiguous choice.
    fn list_conversions(
        &mut self,
        _graph: &Graph,
        _from: PortId,
        _to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        Vec::new()
    }

    /// Returns a UI label for a conversion candidate.
    fn conversion_label(
        &mut self,
        _graph: &Graph,
        _from: PortId,
        _to: PortId,
        template: &InsertNodeTemplate,
    ) -> Arc<str> {
        Arc::<str>::from(format!("Convert: {}", template.kind.0))
    }

    /// Returns where a conversion node should be inserted (canvas space).
    ///
    /// `default_at` is typically the cursor position at drop time.
    fn conversion_insert_position(
        &mut self,
        _graph: &Graph,
        _from: PortId,
        _to: PortId,
        default_at: CanvasPoint,
        _template: &InsertNodeTemplate,
    ) -> CanvasPoint {
        default_at
    }
}

/// Default presenter used by the canvas widget when no domain presenter is provided.
#[derive(Debug, Default, Clone)]
pub struct DefaultNodeGraphPresenter {
    #[cfg(feature = "kit")]
    profile: DataflowProfile,
}

impl NodeGraphPresenter for DefaultNodeGraphPresenter {
    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        graph
            .nodes
            .get(&node)
            .map(|n| Arc::<str>::from(n.kind.0.clone()))
            .unwrap_or_else(|| Arc::<str>::from("<missing node>"))
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        graph
            .ports
            .get(&port)
            .map(|p| Arc::<str>::from(p.key.0.clone()))
            .unwrap_or_else(|| Arc::<str>::from("<missing port>"))
    }

    fn profile_mut(&mut self) -> Option<&mut (dyn GraphProfile + 'static)> {
        #[cfg(feature = "kit")]
        {
            Some(&mut self.profile)
        }
        #[cfg(not(feature = "kit"))]
        {
            None
        }
    }
}

/// Presenter that uses `NodeRegistry` for titles and port labels.
///
/// - `node_title` comes from `NodeSchema.title` (falls back to `Node.kind`).
/// - `port_label` comes from `PortDecl.label` when available; dynamic ports fall back to `Port.key`.
/// - `plan_connect` is profile-driven by default (typed connect + concretize/validate pipeline).
pub struct RegistryNodeGraphPresenter {
    registry: NodeRegistry,
    profile: Option<Box<dyn GraphProfile>>,
}

impl RegistryNodeGraphPresenter {
    pub fn new(registry: NodeRegistry) -> Self {
        Self {
            registry,
            profile: {
                #[cfg(feature = "kit")]
                {
                    Some(Box::new(DataflowProfile::new()))
                }
                #[cfg(not(feature = "kit"))]
                {
                    None
                }
            },
        }
    }

    pub fn with_profile(mut self, profile: impl GraphProfile + 'static) -> Self {
        self.profile = Some(Box::new(profile));
        self
    }

    fn schema_for_node<'a>(
        &'a self,
        graph: &'a Graph,
        node: NodeId,
    ) -> Option<&'a crate::schema::NodeSchema> {
        let n = graph.nodes.get(&node)?;
        self.registry.get(self.registry.resolve_kind(&n.kind))
    }
}

impl NodeGraphPresenter for RegistryNodeGraphPresenter {
    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        self.schema_for_node(graph, node)
            .map(|s| Arc::<str>::from(s.title.clone()))
            .unwrap_or_else(|| {
                graph
                    .nodes
                    .get(&node)
                    .map(|n| Arc::<str>::from(n.kind.0.clone()))
                    .unwrap_or_else(|| Arc::<str>::from("<missing node>"))
            })
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        let Some(p) = graph.ports.get(&port) else {
            return Arc::<str>::from("<missing port>");
        };
        let Some(schema) = self.schema_for_node(graph, p.node) else {
            return Arc::<str>::from(p.key.0.clone());
        };
        schema
            .ports
            .iter()
            .find(|decl| decl.key == p.key)
            .and_then(|decl| decl.label.as_ref())
            .map(|s| Arc::<str>::from(s.clone()))
            .unwrap_or_else(|| Arc::<str>::from(p.key.0.clone()))
    }

    fn profile_mut(&mut self) -> Option<&mut (dyn GraphProfile + 'static)> {
        self.profile
            .as_mut()
            .map(|p| &mut **p as &mut (dyn GraphProfile + 'static))
    }

    fn list_insertable_nodes(&mut self, _graph: &Graph) -> Vec<InsertNodeCandidate> {
        let mut out: Vec<InsertNodeCandidate> = Vec::new();
        for schema in self.registry.schemas() {
            let label = if schema.category.is_empty() {
                schema.title.clone()
            } else {
                format!("{}/{}", schema.category.join("/"), schema.title)
            };
            out.push(InsertNodeCandidate {
                kind: schema.kind.clone(),
                label: Arc::<str>::from(label),
                enabled: true,
                template: None,
                payload: Value::Null,
            });
        }
        out
    }

    fn plan_create_node(
        &mut self,
        _graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        let Some(schema) = self.registry.get(&candidate.kind).cloned() else {
            return Err(Arc::<str>::from("missing node schema"));
        };

        let node_id = NodeId::new();
        let node = Node {
            kind: schema.kind,
            kind_version: schema.latest_kind_version,
            pos: at,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: schema.default_data,
        };

        let mut port_ids: Vec<PortId> = Vec::new();
        let mut ops: Vec<GraphOp> = Vec::new();
        ops.push(GraphOp::AddNode { id: node_id, node });
        for decl in schema.ports {
            let port_id = PortId::new();
            port_ids.push(port_id);
            ops.push(GraphOp::AddPort {
                id: port_id,
                port: Port {
                    node: node_id,
                    key: decl.key,
                    dir: decl.dir,
                    kind: decl.kind,
                    capacity: decl.capacity,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: decl.ty,
                    data: Value::Null,
                },
            });
        }

        ops.push(GraphOp::SetNodePorts {
            id: node_id,
            from: Vec::new(),
            to: port_ids,
        });

        Ok(ops)
    }
}
