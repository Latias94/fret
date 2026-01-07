use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
    run_app,
};
use fret_node::Graph;
use fret_node::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Node, NodeId, NodeKindKey, Port};
use fret_node::core::{PortCapacity, PortDirection, PortId, PortKey, PortKind};
use fret_node::io::NodeGraphViewState;
use fret_node::ops::GraphOp;
use fret_node::rules::{
    ConnectDecision, ConnectPlan, DiagnosticSeverity, DiagnosticTarget, InsertNodeTemplate,
    PortTemplate,
};
use fret_node::types::TypeDesc;
use fret_node::ui::{InsertNodeCandidate, NodeGraphCanvas, NodeGraphPresenter};
use fret_runtime::PlatformCapabilities;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::{UiFrameCx, UiTree};

#[derive(Clone)]
struct NodeGraphDemoModels {
    graph: fret_runtime::Model<Graph>,
    view: fret_runtime::Model<NodeGraphViewState>,
}

fn build_demo_graph() -> Graph {
    let mut graph = Graph::default();

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
        },
    );

    graph
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

fn convert_kind(from: &TypeDesc, to: &TypeDesc) -> Option<NodeKindKey> {
    if is_int(from) && is_float(to) {
        return Some(NodeKindKey::new("demo.convert.int_to_float"));
    }
    if is_float(from) && is_int(to) {
        return Some(NodeKindKey::new("demo.convert.float_to_int"));
    }
    None
}

fn convert_spec(kind: &NodeKindKey) -> Option<(TypeDesc, TypeDesc, Arc<str>)> {
    match kind.0.as_str() {
        "demo.convert.int_to_float" => Some((
            TypeDesc::Int,
            TypeDesc::Float,
            Arc::<str>::from("Convert Int -> Float"),
        )),
        "demo.convert.float_to_int" => Some((
            TypeDesc::Float,
            TypeDesc::Int,
            Arc::<str>::from("Convert Float -> Int"),
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
            .map(|p| {
                let ty = p.ty.as_ref().map(type_name).unwrap_or("?");
                Arc::<str>::from(format!("{}: {}", p.key.0, ty))
            })
            .unwrap_or_else(|| Arc::<str>::from("<missing port>"))
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

        let Some(kind) = convert_kind(from_ty, to_ty) else {
            return Vec::new();
        };

        let Some((from_ty, to_ty, label)) = convert_spec(&kind) else {
            return Vec::new();
        };
        let template = convert_template(&kind, from_ty, to_ty);
        vec![InsertNodeCandidate {
            kind,
            label,
            enabled: true,
            template: Some(template),
            payload: serde_json::Value::Null,
        }]
    }

    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
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
            if let Some(kind) = convert_kind(from_ty, to_ty)
                && convert_spec(&kind).is_some()
            {
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
        let Some(kind) = convert_kind(from_ty, to_ty) else {
            return Vec::new();
        };
        let Some((from_ty, to_ty, _label)) = convert_spec(&kind) else {
            return Vec::new();
        };
        vec![convert_template(&kind, from_ty, to_ty)]
    }
}

struct NodeGraphDomainDemoWindowState {
    ui: UiTree<App>,
    root: fret_core::NodeId,
}

#[derive(Default)]
struct NodeGraphDomainDemoDriver;

impl NodeGraphDomainDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> NodeGraphDomainDemoWindowState {
        let models = app
            .global::<NodeGraphDemoModels>()
            .expect("NodeGraphDemoModels global must exist")
            .clone();

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let presenter = DemoTypedPresenter::default();
        let canvas = NodeGraphCanvas::new(models.graph, models.view).with_presenter(presenter);
        let root = ui.create_node_retained(canvas);
        ui.set_root(root);

        NodeGraphDomainDemoWindowState { ui, root }
    }
}

impl WinitAppDriver for NodeGraphDomainDemoDriver {
    type WindowState = NodeGraphDomainDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        context
            .state
            .ui
            .dispatch_event(context.app, context.services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
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
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<fret_launch::WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
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

    let graph = app.models_mut().insert(build_demo_graph());
    let view = app.models_mut().insert(NodeGraphViewState::default());
    app.set_global(NodeGraphDemoModels { graph, view });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo node_graph_domain_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    run_app(config, app, NodeGraphDomainDemoDriver::default()).map_err(anyhow::Error::from)
}
