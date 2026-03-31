use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Color, Event};
use fret_launch::{
    FnDriver, FnDriverHooks, WindowCreateSpec, WinitCommandContext, WinitEventContext,
    WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_node::Graph;
use fret_node::GraphId;
use fret_node::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Node, NodeId, NodeKindKey, Port};
use fret_node::core::{PortCapacity, PortDirection, PortId, PortKey, PortKind};
use fret_node::io::NodeGraphViewState;
use fret_node::io::NodeGraphViewStateFileV1;
use fret_node::ops::{GraphOp, GraphTransaction};
use fret_node::rules::{
    ConnectDecision, ConnectPlan, DiagnosticSeverity, DiagnosticTarget, InsertNodeTemplate,
    PortTemplate,
};
use fret_node::runtime::callbacks::{
    NodeGraphCommitCallbacks, NodeGraphGestureCallbacks, NodeGraphViewCallbacks,
};
use fret_node::runtime::changes::NodeGraphChanges;
use fret_node::runtime::events::ViewChange;
use fret_node::runtime::store::NodeGraphStore;
use fret_node::types::TypeDesc;
use fret_node::ui::advanced::{NodeGraphControllerTransportExt as _, NodeGraphEditQueue};
use fret_node::ui::canvas::RejectNonFiniteTx;
use fret_node::ui::style::NodeGraphStyle;
use fret_node::ui::{
    EdgeMarker, EdgeRenderHint, EdgeRouteKind, EdgeTypeKey, InsertNodeCandidate,
    MeasuredGeometryStore, NodeGraphA11yFocusedEdge, NodeGraphA11yFocusedNode,
    NodeGraphA11yFocusedPort, NodeGraphCanvas, NodeGraphController, NodeGraphEdgeTypes,
    NodeGraphEditor, NodeGraphInternalsStore, NodeGraphNodeTypes, NodeGraphOverlayHost,
    NodeGraphOverlayState, NodeGraphPortalHost, NodeGraphPortalNodeLayout, NodeGraphPresenter,
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
    register_node_graph_commands,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::Theme;
use fret_ui::retained_bridge::{BoundTextInput, UiTreeRetainedExt as _};
use fret_ui::{UiFrameCx, UiTree};

use fret_app::install_command_default_keybindings_into_keymap;

#[derive(Clone)]
struct NodeGraphDemoModels {
    store: fret_runtime::Model<NodeGraphStore>,
    controller: NodeGraphController,
    graph: fret_runtime::Model<Graph>,
    view: fret_runtime::Model<NodeGraphViewState>,
    edits: fret_runtime::Model<NodeGraphEditQueue>,
    overlays: fret_runtime::Model<NodeGraphOverlayState>,
    group_rename_text: fret_runtime::Model<String>,
}

#[derive(Clone)]
struct NodeGraphDemoViewStatePersistence {
    graph_id: GraphId,
    path: PathBuf,
}

#[derive(Default)]
struct DomainDemoCallbacks {
    commit_count: u64,
    last_viewport_log_at: Option<Instant>,
}

impl NodeGraphCommitCallbacks for DomainDemoCallbacks {
    fn on_graph_commit(&mut self, committed: &GraphTransaction, changes: &NodeGraphChanges) {
        self.commit_count += 1;
        tracing::info!(
            commit = self.commit_count,
            label = committed.label.as_deref().unwrap_or(""),
            ops = committed.ops.len(),
            node_changes = changes.nodes.len(),
            edge_changes = changes.edges.len(),
            "node graph committed"
        );
    }
}

impl NodeGraphViewCallbacks for DomainDemoCallbacks {
    fn on_view_change(&mut self, changes: &[ViewChange]) {
        for change in changes {
            match change {
                ViewChange::Viewport { pan, zoom } => {
                    let now = Instant::now();
                    let should_log = match self.last_viewport_log_at {
                        Some(prev) => now.duration_since(prev) >= Duration::from_millis(200),
                        None => true,
                    };
                    if should_log {
                        self.last_viewport_log_at = Some(now);
                        tracing::info!(
                            pan_x = pan.x,
                            pan_y = pan.y,
                            zoom = *zoom,
                            "viewport changed"
                        );
                    }
                }
                ViewChange::Selection {
                    nodes,
                    edges,
                    groups,
                } => {
                    tracing::info!(
                        nodes = nodes.len(),
                        edges = edges.len(),
                        groups = groups.len(),
                        "selection changed"
                    );
                }
            }
        }
    }
}

impl NodeGraphGestureCallbacks for DomainDemoCallbacks {}

fn build_demo_graph(graph_id: GraphId) -> Graph {
    let mut graph = Graph::new(graph_id);

    let node_int = NodeId::new();
    let node_float = NodeId::new();
    let node_sink = NodeId::new();

    let port_int_out = PortId::new();
    let port_float_out = PortId::new();
    let port_sink_in = PortId::new();

    graph.nodes.insert(
        node_int,
        Node {
            kind: NodeKindKey::new("demo.const_int"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 80.0 },
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
            ports: vec![port_int_out],
            data: serde_json::json!({ "value": 7 }),
        },
    );
    graph.nodes.insert(
        node_float,
        Node {
            kind: NodeKindKey::new("demo.const_float"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 240.0 },
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
            ports: vec![port_float_out],
            data: serde_json::json!({ "value": 0.5 }),
        },
    );
    graph.nodes.insert(
        node_sink,
        Node {
            kind: NodeKindKey::new("demo.sink_float"),
            kind_version: 1,
            pos: CanvasPoint { x: 520.0, y: 160.0 },
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
            ports: vec![port_sink_in],
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_int_out,
        Port {
            node: node_int,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: Some(TypeDesc::Int),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_float_out,
        Port {
            node: node_float,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_sink_in,
        Port {
            node: node_sink,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    // Int -> Float edge (intentionally incompatible) so the demo can showcase split-edge insertion.
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_int_out,
            to: port_sink_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    graph
}

fn set_value_in_node_data(
    mut data: serde_json::Value,
    value: serde_json::Value,
) -> serde_json::Value {
    match &mut data {
        serde_json::Value::Object(map) => {
            map.insert("value".to_string(), value);
            data
        }
        _ => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), value);
            serde_json::Value::Object(map)
        }
    }
}

#[derive(Clone, Copy)]
struct DomainConstNumberSpec;

impl PortalNumberEditSpec for DomainConstNumberSpec {
    fn initial_value(&self, graph: &Graph, node: NodeId) -> Option<f64> {
        let node = graph.nodes.get(&node)?;
        match node.kind.0.as_str() {
            "demo.const_int" => node.data.get("value")?.as_i64().map(|v| v as f64),
            "demo.const_float" => node.data.get("value")?.as_f64(),
            _ => None,
        }
    }

    fn round_value(&self, graph: &Graph, node: NodeId, value: f64) -> f64 {
        let Some(node) = graph.nodes.get(&node) else {
            return value;
        };
        match node.kind.0.as_str() {
            "demo.const_int" => value.round(),
            _ => value,
        }
    }

    fn submit_value(
        &self,
        graph: &Graph,
        node: NodeId,
        value: f64,
        _text: &str,
    ) -> PortalNumberEditSubmit {
        let Some(node_data) = graph.nodes.get(&node) else {
            return PortalNumberEditSubmit::NotHandled;
        };

        let (to_value, normalized) = match node_data.kind.0.as_str() {
            "demo.const_int" => {
                let v = value.round() as i64;
                (serde_json::Value::from(v), Some(v.to_string()))
            }
            "demo.const_float" => (serde_json::Value::from(value), Some(format!("{value}"))),
            _ => return PortalNumberEditSubmit::NotHandled,
        };

        let from = node_data.data.clone();
        let to = set_value_in_node_data(from.clone(), to_value);

        PortalNumberEditSubmit::Commit {
            tx: fret_node::ops::GraphTransaction {
                label: Some("Set Const Value".to_string()),
                ops: vec![GraphOp::SetNodeData { id: node, from, to }],
            },
            normalized_text: normalized,
        }
    }

    fn supports_drag(&self, graph: &Graph, node: NodeId) -> bool {
        graph
            .nodes
            .get(&node)
            .is_some_and(|n| matches!(n.kind.0.as_str(), "demo.const_int" | "demo.const_float"))
    }

    fn drag_threshold_px(&self, _graph: &Graph, _node: NodeId) -> f32 {
        2.0
    }
}

fn is_int(t: &TypeDesc) -> bool {
    matches!(t, TypeDesc::Int)
}

fn is_float(t: &TypeDesc) -> bool {
    matches!(t, TypeDesc::Float)
}

fn type_name(t: &TypeDesc) -> &'static str {
    match t {
        TypeDesc::Int => "Int",
        TypeDesc::Float => "Float",
        TypeDesc::Bool => "Bool",
        TypeDesc::String => "String",
        TypeDesc::Any => "Any",
        TypeDesc::Unknown => "Unknown",
        TypeDesc::Null => "Null",
        _ => "Other",
    }
}

fn convert_kinds(from: &TypeDesc, to: &TypeDesc) -> Vec<NodeKindKey> {
    if is_int(from) && is_float(to) {
        return vec![
            NodeKindKey::new("demo.convert.int_to_float.cast"),
            NodeKindKey::new("demo.convert.int_to_float.exact"),
        ];
    }
    if is_float(from) && is_int(to) {
        return vec![
            NodeKindKey::new("demo.convert.float_to_int.truncate"),
            NodeKindKey::new("demo.convert.float_to_int.round"),
        ];
    }
    Vec::new()
}

fn convert_spec(kind: &NodeKindKey) -> Option<(TypeDesc, TypeDesc, Arc<str>)> {
    match kind.0.as_str() {
        "demo.convert.int_to_float.cast" => Some((
            TypeDesc::Int,
            TypeDesc::Float,
            Arc::<str>::from("Cast Int -> Float"),
        )),
        "demo.convert.int_to_float.exact" => Some((
            TypeDesc::Int,
            TypeDesc::Float,
            Arc::<str>::from("Exact Int -> Float"),
        )),
        "demo.convert.float_to_int.truncate" => Some((
            TypeDesc::Float,
            TypeDesc::Int,
            Arc::<str>::from("Truncate Float -> Int"),
        )),
        "demo.convert.float_to_int.round" => Some((
            TypeDesc::Float,
            TypeDesc::Int,
            Arc::<str>::from("Round Float -> Int"),
        )),
        _ => None,
    }
}

fn convert_template(kind: &NodeKindKey, from_ty: TypeDesc, to_ty: TypeDesc) -> InsertNodeTemplate {
    InsertNodeTemplate {
        kind: kind.clone(),
        kind_version: 1,
        collapsed: false,
        data: serde_json::Value::Null,
        ports: vec![
            PortTemplate {
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(from_ty),
                data: serde_json::Value::Null,
            },
            PortTemplate {
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: Some(to_ty),
                data: serde_json::Value::Null,
            },
        ],
        input: PortKey::new("in"),
        output: PortKey::new("out"),
    }
}

#[derive(Debug, Default, Clone)]
struct DemoTypedPresenter;

impl NodeGraphPresenter for DemoTypedPresenter {
    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        let Some(n) = graph.nodes.get(&node) else {
            return Arc::<str>::from("<missing node>");
        };

        if let Some((_from, _to, label)) = convert_spec(&n.kind) {
            return label;
        }

        match n.kind.0.as_str() {
            "demo.const_int" => Arc::<str>::from("Const Int"),
            "demo.const_float" => Arc::<str>::from("Const Float"),
            "demo.sink_float" => Arc::<str>::from("Sink (Float)"),
            _ => Arc::<str>::from(n.kind.0.clone()),
        }
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        graph
            .ports
            .get(&port)
            .map(|p| {
                let ty = p.ty.as_ref().map(type_name).unwrap_or("?");
                Arc::<str>::from(format!("{}: {}", p.key.0, ty))
            })
            .unwrap_or_else(|| Arc::<str>::from("<missing port>"))
    }

    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        _style: &fret_node::ui::NodeGraphStyle,
    ) -> EdgeRenderHint {
        let Some(e) = graph.edges.get(&edge) else {
            return EdgeRenderHint {
                width_mul: 1.0,
                ..EdgeRenderHint::default()
            };
        };

        let mut hint = EdgeRenderHint {
            width_mul: 1.0,
            ..EdgeRenderHint::default()
        };

        if e.kind == EdgeKind::Exec {
            hint.route = EdgeRouteKind::Step;
            hint.end_marker = Some(EdgeMarker::arrow(12.0));
            return hint;
        }

        let from_ty = graph.ports.get(&e.from).and_then(|p| p.ty.as_ref());
        let to_ty = graph.ports.get(&e.to).and_then(|p| p.ty.as_ref());
        if let (Some(from_ty), Some(to_ty)) = (from_ty, to_ty) {
            let from_name = type_name(from_ty);
            let to_name = type_name(to_ty);
            if from_name != to_name {
                hint.label = Some(Arc::<str>::from(format!("{from_name}→{to_name}")));
            } else {
                hint.label = Some(Arc::<str>::from(from_name.to_string()));
            }
        }

        hint
    }

    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        let Some(e) = graph.edges.get(&edge) else {
            return Vec::new();
        };
        let from = graph.ports.get(&e.from);
        let to = graph.ports.get(&e.to);
        let (Some(from), Some(to)) = (from, to) else {
            return Vec::new();
        };
        let (Some(from_ty), Some(to_ty)) = (from.ty.as_ref(), to.ty.as_ref()) else {
            return Vec::new();
        };

        let kinds = convert_kinds(from_ty, to_ty);
        let mut out: Vec<InsertNodeCandidate> = Vec::new();
        for kind in kinds {
            let Some((from_ty, to_ty, label)) = convert_spec(&kind) else {
                continue;
            };
            let template = convert_template(&kind, from_ty, to_ty);
            out.push(InsertNodeCandidate {
                kind,
                label,
                enabled: true,
                template: Some(template),
                payload: serde_json::Value::Null,
            });
        }
        out
    }

    fn plan_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        _mode: fret_node::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        let Some(port_a) = graph.ports.get(&a) else {
            return ConnectPlan::reject("missing port");
        };
        let Some(port_b) = graph.ports.get(&b) else {
            return ConnectPlan::reject("missing port");
        };
        if port_a.kind != PortKind::Data || port_b.kind != PortKind::Data {
            return ConnectPlan::reject("demo only supports data ports");
        }

        // Orient (out -> in).
        let (from, to) = match (port_a.dir, port_b.dir) {
            (PortDirection::Out, PortDirection::In) => (a, b),
            (PortDirection::In, PortDirection::Out) => (b, a),
            _ => {
                return ConnectPlan::reject("ports must be out -> in");
            }
        };

        if from == to {
            return ConnectPlan::reject("cannot connect a port to itself");
        }
        let from_port = graph.ports.get(&from).expect("checked");
        let to_port = graph.ports.get(&to).expect("checked");
        if from_port.node == to_port.node {
            return ConnectPlan::reject("cannot connect ports on the same node");
        }

        let from_ty = from_port.ty.as_ref();
        let to_ty = to_port.ty.as_ref();

        if let (Some(from_ty), Some(to_ty)) = (from_ty, to_ty) {
            if !convert_kinds(from_ty, to_ty).is_empty() {
                return ConnectPlan {
                    decision: ConnectDecision::Reject,
                    diagnostics: vec![fret_node::rules::Diagnostic {
                        key: "demo.convertible".to_string(),
                        severity: DiagnosticSeverity::Warning,
                        target: DiagnosticTarget::Graph,
                        message: format!(
                            "type mismatch: {} -> {} (conversion available)",
                            type_name(from_ty),
                            type_name(to_ty)
                        ),
                        fixes: Vec::new(),
                    }],
                    ops: Vec::new(),
                };
            }

            if from_ty != to_ty {
                return ConnectPlan {
                    decision: ConnectDecision::Reject,
                    diagnostics: vec![fret_node::rules::Diagnostic {
                        key: "demo.type_mismatch".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: DiagnosticTarget::Graph,
                        message: format!(
                            "type mismatch: {} -> {}",
                            type_name(from_ty),
                            type_name(to_ty)
                        ),
                        fixes: Vec::new(),
                    }],
                    ops: Vec::new(),
                };
            }
        }

        // Capacity handling (mimic the rule layer).
        let mut ops: Vec<GraphOp> = Vec::new();
        if from_port.capacity == PortCapacity::Single {
            for (edge_id, edge) in &graph.edges {
                if edge.from == from {
                    ops.push(GraphOp::RemoveEdge {
                        id: *edge_id,
                        edge: edge.clone(),
                    });
                }
            }
        }
        if to_port.capacity == PortCapacity::Single {
            for (edge_id, edge) in &graph.edges {
                if edge.to == to {
                    ops.push(GraphOp::RemoveEdge {
                        id: *edge_id,
                        edge: edge.clone(),
                    });
                }
            }
        }

        ops.push(GraphOp::AddEdge {
            id: EdgeId::new(),
            edge: Edge {
                kind: EdgeKind::Data,
                from,
                to,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        });

        ConnectPlan {
            decision: ConnectDecision::Accept,
            diagnostics: Vec::new(),
            ops,
        }
    }

    fn list_conversions(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        let Some(from_port) = graph.ports.get(&from) else {
            return Vec::new();
        };
        let Some(to_port) = graph.ports.get(&to) else {
            return Vec::new();
        };
        if from_port.kind != PortKind::Data || to_port.kind != PortKind::Data {
            return Vec::new();
        }
        let (from_ty, to_ty) = match (from_port.ty.as_ref(), to_port.ty.as_ref()) {
            (Some(a), Some(b)) => (a, b),
            _ => return Vec::new(),
        };
        let kinds = convert_kinds(from_ty, to_ty);
        let mut out: Vec<InsertNodeTemplate> = Vec::new();
        for kind in kinds {
            let Some((from_ty, to_ty, _label)) = convert_spec(&kind) else {
                continue;
            };
            out.push(convert_template(&kind, from_ty, to_ty));
        }
        out
    }

    fn conversion_label(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        template: &InsertNodeTemplate,
    ) -> Arc<str> {
        if let Some((_from_ty, _to_ty, label)) = convert_spec(&template.kind) {
            return label;
        }

        let from_ty = graph.ports.get(&from).and_then(|p| p.ty.as_ref());
        let to_ty = graph.ports.get(&to).and_then(|p| p.ty.as_ref());
        if let (Some(from_ty), Some(to_ty)) = (from_ty, to_ty) {
            return Arc::<str>::from(format!("{} -> {}", type_name(from_ty), type_name(to_ty)));
        }

        Arc::<str>::from(format!("Convert: {}", template.kind.0))
    }

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

struct NodeGraphDomainDemoWindowState {
    ui: UiTree<App>,
    root: fret_core::NodeId,
}

#[derive(Default)]
struct NodeGraphDomainDemoDriver {
    pending_view_state_save: bool,
    last_view_state_save_at: Option<Instant>,
}

impl NodeGraphDomainDemoDriver {
    const VIEW_STATE_SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

    fn save_view_state(&mut self, app: &mut App) {
        let Some(models) = app.global::<NodeGraphDemoModels>() else {
            return;
        };
        let Some(persist) = app.global::<NodeGraphDemoViewStatePersistence>() else {
            return;
        };

        let Ok(state) = models.store.read_ref(app, |s| s.view_state().clone()) else {
            return;
        };

        let file = NodeGraphViewStateFileV1::new(persist.graph_id, state);
        if let Err(err) = file.save_json(&persist.path) {
            tracing::warn!(?err, "failed to save node graph view state");
        }
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> NodeGraphDomainDemoWindowState {
        let models = app
            .global::<NodeGraphDemoModels>()
            .expect("NodeGraphDemoModels global must exist")
            .clone();
        let internals = app
            .global::<Arc<NodeGraphInternalsStore>>()
            .expect("NodeGraphInternalsStore global must exist")
            .clone();
        let measured = app
            .global::<Arc<MeasuredGeometryStore>>()
            .expect("MeasuredGeometryStore global must exist")
            .clone();

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let graph = models.graph.clone();
        let view = models.view.clone();
        let overlays = models.overlays.clone();
        let group_rename_text = models.group_rename_text.clone();
        let controller = models.controller.clone();
        let style = NodeGraphStyle::from_theme(Theme::global(app));

        let edge_types = NodeGraphEdgeTypes::new()
            .with_resolver(|graph, edge_id| {
                let Some(edge) = graph.edges.get(&edge_id) else {
                    return EdgeTypeKey::new("data");
                };
                if edge.kind == EdgeKind::Exec {
                    return EdgeTypeKey::new("exec");
                }

                let from_ty = graph.ports.get(&edge.from).and_then(|p| p.ty.as_ref());
                let to_ty = graph.ports.get(&edge.to).and_then(|p| p.ty.as_ref());
                if let (Some(from_ty), Some(to_ty)) = (from_ty, to_ty) {
                    if from_ty != to_ty {
                        return EdgeTypeKey::new("incompatible");
                    }
                }

                EdgeTypeKey::new("data")
            })
            .register(
                EdgeTypeKey::new("incompatible"),
                |_graph, _edge, _style, mut hint| {
                    hint.color = Some(Color {
                        r: 0.92,
                        g: 0.25,
                        b: 0.25,
                        a: 1.0,
                    });
                    hint.label = Some(Arc::<str>::from("Type mismatch"));
                    hint.width_mul = 1.25;
                    hint.end_marker = Some(EdgeMarker::arrow(12.0));
                    hint.route = EdgeRouteKind::Step;
                    hint
                },
            )
            .register(
                EdgeTypeKey::new("exec"),
                |_graph, _edge, _style, mut hint| {
                    hint.width_mul = 1.1;
                    hint.end_marker = Some(EdgeMarker::arrow(12.0));
                    hint.route = EdgeRouteKind::Step;
                    hint
                },
            );

        let presenter = DemoTypedPresenter::default();
        let canvas = NodeGraphCanvas::new(graph.clone(), view)
            .with_controller(controller.clone())
            .with_middleware(RejectNonFiniteTx)
            .with_presenter(presenter)
            .with_style(style.clone())
            .with_edge_types(edge_types)
            .with_callbacks(DomainDemoCallbacks::default())
            .with_overlay_state(overlays.clone())
            .with_internals_store(internals.clone())
            .with_close_command(CommandId::new("node_graph_domain_demo.close"));
        let canvas_node = ui.create_node_retained(canvas);

        let a11y_port = ui.create_node_retained(NodeGraphA11yFocusedPort::new(internals.clone()));
        let a11y_edge = ui.create_node_retained(NodeGraphA11yFocusedEdge::new(internals.clone()));
        let a11y_node = ui.create_node_retained(NodeGraphA11yFocusedNode::new(internals));
        ui.set_children(canvas_node, vec![a11y_port, a11y_edge, a11y_node]);

        let overlay_host = NodeGraphOverlayHost::new(
            graph,
            overlays,
            group_rename_text.clone(),
            canvas_node,
            style.clone(),
        )
        .with_controller(controller.clone());
        let overlay_node = ui.create_node_retained(overlay_host);
        let rename_input_node = ui.create_node_retained(BoundTextInput::new(group_rename_text));
        ui.set_children(overlay_node, vec![rename_input_node]);

        let portal_root = "node_graph_domain_demo.portal";
        let portal_editor = PortalNumberEditor::new(portal_root);

        let node_types = NodeGraphNodeTypes::new()
            .register(NodeKindKey::new("demo.const_int"), {
                let portal_editor = portal_editor.clone();
                let portal_graph_model = models.graph.clone();
                let style = style.clone();
                move |ecx, graph, layout: NodeGraphPortalNodeLayout| {
                    portal_editor.render_number_input_for_node(
                        ecx,
                        portal_graph_model.clone(),
                        graph,
                        layout,
                        &style,
                        layout.node,
                        &DomainConstNumberSpec,
                    )
                }
            })
            .register(NodeKindKey::new("demo.const_float"), {
                let portal_editor = portal_editor.clone();
                let portal_graph_model = models.graph.clone();
                let style = style.clone();
                move |ecx, graph, layout: NodeGraphPortalNodeLayout| {
                    portal_editor.render_number_input_for_node(
                        ecx,
                        portal_graph_model.clone(),
                        graph,
                        layout,
                        &style,
                        layout.node,
                        &DomainConstNumberSpec,
                    )
                }
            });

        let portal = NodeGraphPortalHost::new(
            models.graph.clone(),
            models.view.clone(),
            measured,
            style.clone(),
            portal_root,
            node_types.into_portal_renderer(),
        )
        .with_cull_margin_px(style.paint.render_cull_margin_px)
        .with_controller(models.controller.clone())
        .with_canvas_focus_target(canvas_node)
        .with_command_handler(PortalNumberEditHandler::new(
            portal_root,
            DomainConstNumberSpec,
        ));
        let portal_node = ui.create_node_retained(portal);

        let root = ui.create_node_retained(NodeGraphEditor::new());
        ui.set_children(root, vec![canvas_node, portal_node, overlay_node]);
        ui.set_root(root);

        NodeGraphDomainDemoWindowState { ui, root }
    }
}

fn create_window_state(
    driver: &mut NodeGraphDomainDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> NodeGraphDomainDemoWindowState {
    NodeGraphDomainDemoDriver::build_ui(app, window)
}

fn handle_model_changes(
    driver: &mut NodeGraphDomainDemoDriver,
    context: WinitWindowContext<'_, NodeGraphDomainDemoWindowState>,
    changed: &[fret_app::ModelId],
) {
    context
        .state
        .ui
        .propagate_model_changes(context.app, changed);

    let Some(models) = context.app.global::<NodeGraphDemoModels>() else {
        return;
    };
    if changed.contains(&models.view.id()) {
        driver.pending_view_state_save = true;
    }
    if driver.pending_view_state_save {
        let now = Instant::now();
        let due = driver.last_view_state_save_at.map_or(true, |t| {
            now.duration_since(t) >= NodeGraphDomainDemoDriver::VIEW_STATE_SAVE_DEBOUNCE
        });
        if due {
            driver.pending_view_state_save = false;
            driver.last_view_state_save_at = Some(now);
            driver.save_view_state(context.app);
        }
    }
}

fn handle_global_changes(
    driver: &mut NodeGraphDomainDemoDriver,
    context: WinitWindowContext<'_, NodeGraphDomainDemoWindowState>,
    changed: &[std::any::TypeId],
) {
    context
        .state
        .ui
        .propagate_global_changes(context.app, changed);
}

fn handle_event(
    driver: &mut NodeGraphDomainDemoDriver,
    context: WinitEventContext<'_, NodeGraphDomainDemoWindowState>,
    event: &Event,
) {
    let WinitEventContext {
        app,
        services,
        window,
        state,
    } = context;

    if matches!(event, Event::WindowCloseRequested) {
        driver.save_view_state(app);
        app.push_effect(Effect::Window(WindowRequest::Close(window)));
        return;
    }

    if let Event::KeyDown { key, .. } = event {
        if *key == fret_core::KeyCode::Escape {
            driver.save_view_state(app);
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }
    }

    state.ui.dispatch_event(app, services, event);
}

fn handle_command(
    driver: &mut NodeGraphDomainDemoDriver,
    context: WinitCommandContext<'_, NodeGraphDomainDemoWindowState>,
    command: CommandId,
) {
    let WinitCommandContext {
        app,
        services,
        window,
        state,
    } = context;

    if state.ui.dispatch_command(app, services, &command) {
        return;
    }

    if command.as_str() == "node_graph_domain_demo.close" {
        driver.save_view_state(app);
        app.push_effect(Effect::Window(WindowRequest::Close(window)));
    }
}

fn render(
    driver: &mut NodeGraphDomainDemoDriver,
    context: WinitRenderContext<'_, NodeGraphDomainDemoWindowState>,
) {
    let WinitRenderContext {
        app,
        services,
        window,
        state,
        bounds,
        scale_factor,
        scene,
    } = context;

    state.ui.set_root(state.root);
    state.ui.ingest_paint_cache_source(scene);
    scene.clear();

    let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    frame.paint_all(scene);
}

fn window_create_spec(
    driver: &mut NodeGraphDomainDemoDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    driver: &mut NodeGraphDomainDemoDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
    new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut FnDriverHooks<NodeGraphDomainDemoDriver, NodeGraphDomainDemoWindowState>,
) {
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
    hooks.window_created = Some(window_created);
}

pub fn build_fn_driver() -> FnDriver<NodeGraphDomainDemoDriver, NodeGraphDomainDemoWindowState> {
    FnDriver::new(
        NodeGraphDomainDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    register_node_graph_commands(app.commands_mut());
    install_command_default_keybindings_into_keymap(&mut app);

    let graph_id = GraphId::from_u128(0x1350_0000_0000_0000_0000_0000_0000_00A2);
    let graph_value = build_demo_graph(graph_id);
    let view_state_path = fret_node::io::default_project_view_state_path(graph_value.graph_id);
    let mut view_value =
        match NodeGraphViewStateFileV1::load_json_if_exists(&view_state_path, graph_value.graph_id)
        {
            Ok(Some(file)) => file.state,
            Ok(None) => NodeGraphViewState::default(),
            Err(err) => {
                tracing::warn!(?err, "failed to load node graph view state; using defaults");
                NodeGraphViewState::default()
            }
        };
    view_value.sanitize_for_graph(&graph_value);

    let store_value = NodeGraphStore::new(graph_value, view_value);
    let graph = app.models_mut().insert(store_value.graph().clone());
    let view = app.models_mut().insert(store_value.view_state().clone());
    let store = app.models_mut().insert(store_value);
    let edits = app.models_mut().insert(NodeGraphEditQueue::default());
    let controller =
        NodeGraphController::new(store.clone()).bind_edit_queue_transport(edits.clone());
    let overlays = app.models_mut().insert(NodeGraphOverlayState::default());
    let group_rename_text = app.models_mut().insert(String::new());
    app.set_global(NodeGraphDemoModels {
        store,
        controller,
        graph,
        view,
        edits,
        overlays,
        group_rename_text,
    });
    app.set_global(NodeGraphDemoViewStatePersistence {
        graph_id,
        path: view_state_path,
    });
    app.set_global(Arc::new(NodeGraphInternalsStore::new()));
    app.set_global(Arc::new(MeasuredGeometryStore::new()));

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo node_graph_domain_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    fret::advanced::run_native_with_configured_fn_driver(config, app, build_fn_driver())
        .map_err(anyhow::Error::from)
}
