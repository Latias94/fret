use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use fret_app::App;
use fret_app::{CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, KeyCode, Modifiers, Point, Px, Rect, Size};
use fret_launch::{
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext, run_app,
};
use fret_runtime::PlatformCapabilities;
use fret_runtime::{
    CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, KeyChord, KeymapService,
    PlatformFilter, WhenExpr, keymap::Binding,
};
use fret_ui::Theme;
use fret_ui::retained_bridge::{BoundTextInput, UiTreeRetainedExt as _};
use fret_ui::{UiFrameCx, UiTree};
use serde_json::Value;

use crate::node_graph_tuning_overlay::NodeGraphTuningOverlay;

use fret_node::Graph;
use fret_node::TypeDesc;
use fret_node::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Node, NodeId, NodeKindKey, Port};
use fret_node::core::{PortCapacity, PortDirection, PortId, PortKey, PortKind};
use fret_node::io::NodeGraphViewState;
use fret_node::ops::{GraphOp, GraphTransaction};
use fret_node::profile::{DataflowProfile, apply_transaction_with_profile};
use fret_node::schema::{NodeRegistry, NodeSchema, PortDecl};
use fret_node::ui::presenter::{
    EdgeRenderHint, EdgeRouteKind, InsertNodeCandidate, NodeGraphContextMenuItem,
    NodeGraphPresenter, PortAnchorHint,
};
use fret_node::ui::style::NodeGraphStyle;
use fret_node::ui::{
    MeasuredGeometryStore, MeasuredNodeGraphPresenter, NodeGraphCanvas, NodeGraphControlsOverlay,
    NodeGraphEditQueue, NodeGraphEditor, NodeGraphInternalsStore, NodeGraphMiniMapOverlay,
    NodeGraphOverlayHost, NodeGraphOverlayState, NodeGraphPortalHost, NodeGraphPortalNodeLayout,
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
    RegistryNodeGraphPresenter, register_node_graph_commands,
};
use fret_ui::element::AnyElement;

#[derive(Clone)]
struct NodeGraphDemoModels {
    graph: fret_runtime::Model<Graph>,
    view: fret_runtime::Model<NodeGraphViewState>,
    edits: fret_runtime::Model<NodeGraphEditQueue>,
    overlays: fret_runtime::Model<NodeGraphOverlayState>,
    group_rename_text: fret_runtime::Model<String>,
}

const CMD_TOGGLE_WEIRD_LAYOUT: &str = "node_graph_demo.toggle_weird_layout";
const CMD_LOG_INTERNALS: &str = "node_graph_demo.log_internals";
const CMD_LOG_MEASURED: &str = "node_graph_demo.log_measured";
const CMD_BUMP_FLOAT_VALUE: &str = "node_graph_demo.bump_float_value";
const WEIRD_KIND: &str = "demo.weird_layout";

#[derive(Clone)]
struct NodeGraphDemoMeasuredStores {
    manual: Arc<MeasuredGeometryStore>,
    derived: Arc<MeasuredGeometryStore>,
}

#[derive(Debug)]
struct DemoWeirdLayoutMeasuredState {
    enabled: AtomicBool,
}

impl DemoWeirdLayoutMeasuredState {
    fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }

    fn toggle(&self) -> bool {
        let next = !self.enabled.load(Ordering::Relaxed);
        self.enabled.store(next, Ordering::Relaxed);
        next
    }
}

struct DemoPresenter {
    inner: RegistryNodeGraphPresenter,
}

impl DemoPresenter {
    fn new(registry: NodeRegistry) -> Self {
        Self {
            inner: RegistryNodeGraphPresenter::new(registry),
        }
    }

    fn is_weird(graph: &Graph, node: NodeId) -> bool {
        graph
            .nodes
            .get(&node)
            .is_some_and(|n| n.kind.0 == WEIRD_KIND)
    }

    fn weird_size_px(&self) -> (f32, f32) {
        (280.0, 240.0)
    }

    fn weird_anchor_for_key(
        mode_b: bool,
        key: &str,
        (w, h): (f32, f32),
        pin_radius: f32,
    ) -> Option<PortAnchorHint> {
        let r = pin_radius.max(1.0);
        let (cx, cy) = if mode_b {
            match key {
                "in_a" => (w * 0.22, 18.0),
                "in_b" => (w * 0.62, 18.0),
                "out_main" => (w - 18.0, h * 0.50),
                "out_aux" => (w * 0.50, h - 18.0),
                _ => return None,
            }
        } else {
            match key {
                "in_a" => (18.0, h * 0.22),
                "in_b" => (18.0, h * 0.72),
                "out_main" => (w - 18.0, h * 0.35),
                "out_aux" => (w - 42.0, h * 0.80),
                _ => return None,
            }
        };

        let center = Point::new(Px(cx), Px(cy));
        let bounds = Rect::new(
            Point::new(Px(cx - r), Px(cy - r)),
            Size::new(Px(2.0 * r), Px(2.0 * r)),
        );
        Some(PortAnchorHint { center, bounds })
    }
}

impl NodeGraphPresenter for DemoPresenter {
    fn geometry_revision(&self) -> u64 {
        1
    }

    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        self.inner.node_title(graph, node)
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        self.inner.port_label(graph, port)
    }

    fn node_body_label(&self, graph: &Graph, node: NodeId) -> Option<Arc<str>> {
        let n = graph.nodes.get(&node)?;
        if n.kind.0.as_str() != "demo.float" {
            return None;
        }
        let value = n.data.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Some(Arc::<str>::from(format!("Value: {:.3}", value)))
    }

    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        _style: &NodeGraphStyle,
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
        match e.kind {
            fret_node::core::EdgeKind::Exec => {
                hint.route = EdgeRouteKind::Step;
            }
            fret_node::core::EdgeKind::Data => {
                let ty = graph
                    .ports
                    .get(&e.from)
                    .and_then(|p| p.ty.as_ref())
                    .or_else(|| graph.ports.get(&e.to).and_then(|p| p.ty.as_ref()));
                if let Some(ty) = ty {
                    let s = match ty {
                        TypeDesc::Any => "any".to_string(),
                        TypeDesc::Unknown => "unknown".to_string(),
                        TypeDesc::Never => "never".to_string(),
                        TypeDesc::Null => "null".to_string(),
                        TypeDesc::Bool => "bool".to_string(),
                        TypeDesc::Int => "int".to_string(),
                        TypeDesc::Float => "float".to_string(),
                        TypeDesc::String => "string".to_string(),
                        TypeDesc::Bytes => "bytes".to_string(),
                        TypeDesc::List { of } => format!("list<{:?}>", of),
                        TypeDesc::Map { .. } => "map".to_string(),
                        TypeDesc::Object { .. } => "object".to_string(),
                        TypeDesc::Union { .. } => "union".to_string(),
                        TypeDesc::Option { of } => format!("option<{:?}>", of),
                        TypeDesc::Var { id } => format!("t{}", id.0),
                        TypeDesc::Opaque { key, .. } => key.clone(),
                    };
                    hint.label = Some(Arc::<str>::from(s));
                }
            }
        }
        hint
    }

    fn list_insertable_nodes(&mut self, graph: &Graph) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes(graph)
    }

    fn plan_create_node(
        &mut self,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<fret_node::ops::GraphOp>, Arc<str>> {
        self.inner.plan_create_node(graph, candidate, at)
    }

    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        self.inner.fill_edge_context_menu(graph, edge, style, out)
    }

    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<fret_node::ops::GraphOp>> {
        self.inner.on_edge_context_menu_action(graph, edge, action)
    }

    fn profile_mut(&mut self) -> Option<&mut (dyn fret_node::profile::GraphProfile + 'static)> {
        self.inner.profile_mut()
    }

    fn node_size_hint_px(
        &mut self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
    ) -> Option<(f32, f32)> {
        if Self::is_weird(graph, node) {
            return Some(self.weird_size_px());
        }

        let Some(n) = graph.nodes.get(&node) else {
            return None;
        };
        if n.kind.0.as_str() == "demo.float" {
            let extra_body = 30.0;
            let pins = style.pin_row_height;
            let base = style.node_header_height + 2.0 * style.node_padding + pins;
            return Some((style.node_width, base + extra_body));
        }

        None
    }

    fn port_anchor_hint(
        &mut self,
        graph: &Graph,
        node: NodeId,
        port: PortId,
        style: &NodeGraphStyle,
    ) -> Option<PortAnchorHint> {
        if !Self::is_weird(graph, node) {
            return None;
        }

        let p = graph.ports.get(&port)?;
        let key = p.key.0.as_str();
        let (w, h) = self.weird_size_px();
        Self::weird_anchor_for_key(false, key, (w, h), style.pin_radius)
    }
}

fn build_demo_registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();

    reg.register(NodeSchema {
        kind: NodeKindKey::new("demo.float"),
        latest_kind_version: 1,
        kind_aliases: Vec::new(),
        title: "Float".to_string(),
        category: vec!["Demo".to_string()],
        keywords: vec!["number".to_string(), "float".to_string()],
        ports: vec![PortDecl {
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            label: Some("Out".to_string()),
        }],
        default_data: serde_json::Value::Null,
    });

    reg.register(NodeSchema {
        kind: NodeKindKey::new("fret.variadic_merge"),
        latest_kind_version: 1,
        kind_aliases: Vec::new(),
        title: "Variadic Merge".to_string(),
        category: vec!["Fret".to_string(), "Graph".to_string()],
        keywords: vec!["variadic".to_string(), "merge".to_string()],
        ports: vec![PortDecl {
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            label: Some("Out".to_string()),
        }],
        default_data: serde_json::Value::Null,
    });

    reg.register(NodeSchema {
        kind: NodeKindKey::new("demo.add"),
        latest_kind_version: 1,
        kind_aliases: Vec::new(),
        title: "Add".to_string(),
        category: vec!["Demo".to_string(), "Math".to_string()],
        keywords: vec!["add".to_string(), "+".to_string()],
        ports: vec![
            PortDecl {
                key: PortKey::new("a"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("A".to_string()),
            },
            PortDecl {
                key: PortKey::new("b"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("B".to_string()),
            },
            PortDecl {
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("Out".to_string()),
            },
        ],
        default_data: serde_json::Value::Null,
    });

    reg.register(NodeSchema {
        kind: NodeKindKey::new("demo.output"),
        latest_kind_version: 1,
        kind_aliases: Vec::new(),
        title: "Output".to_string(),
        category: vec!["Demo".to_string()],
        keywords: vec!["sink".to_string(), "output".to_string()],
        ports: vec![PortDecl {
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            label: Some("In".to_string()),
        }],
        default_data: serde_json::Value::Null,
    });

    reg.register(NodeSchema {
        kind: NodeKindKey::new(WEIRD_KIND),
        latest_kind_version: 1,
        kind_aliases: Vec::new(),
        title: "Weird Layout".to_string(),
        category: vec!["Demo".to_string(), "Layout".to_string()],
        keywords: vec![
            "weird".to_string(),
            "layout".to_string(),
            "anchors".to_string(),
            "measured".to_string(),
        ],
        ports: vec![
            PortDecl {
                key: PortKey::new("in_a"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("In A".to_string()),
            },
            PortDecl {
                key: PortKey::new("in_b"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("In B".to_string()),
            },
            PortDecl {
                key: PortKey::new("out_main"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("Main".to_string()),
            },
            PortDecl {
                key: PortKey::new("out_aux"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: Some(TypeDesc::Float),
                label: Some("Aux".to_string()),
            },
        ],
        default_data: serde_json::Value::Null,
    });

    reg
}

fn build_demo_graph() -> Graph {
    let mut graph = Graph::default();

    let node_value_a = NodeId::new();
    let node_value_b = NodeId::new();
    let node_merge = NodeId::new();
    let node_add = NodeId::new();
    let node_out = NodeId::new();
    let node_weird = NodeId::new();

    let port_value_a_out = PortId::new();
    let port_value_b_out = PortId::new();
    let port_merge_in0 = PortId::new();
    let port_merge_in1 = PortId::new();
    let port_merge_out = PortId::new();
    let port_add_a = PortId::new();
    let port_add_b = PortId::new();
    let port_add_out = PortId::new();
    let port_out_in = PortId::new();
    let port_weird_in_a = PortId::new();
    let port_weird_in_b = PortId::new();
    let port_weird_out_main = PortId::new();
    let port_weird_out_aux = PortId::new();

    graph.nodes.insert(
        node_value_a,
        Node {
            kind: NodeKindKey::new("demo.float"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 60.0 },
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![port_value_a_out],
            data: serde_json::json!({ "value": 0.25 }),
        },
    );
    graph.nodes.insert(
        node_value_b,
        Node {
            kind: NodeKindKey::new("demo.float"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 170.0 },
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![port_value_b_out],
            data: serde_json::json!({ "value": 0.75 }),
        },
    );
    graph.nodes.insert(
        node_merge,
        Node {
            kind: NodeKindKey::new("fret.variadic_merge"),
            kind_version: 1,
            pos: CanvasPoint { x: 300.0, y: 90.0 },
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![port_merge_in0, port_merge_in1, port_merge_out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_add,
        Node {
            kind: NodeKindKey::new("demo.add"),
            kind_version: 1,
            pos: CanvasPoint { x: 560.0, y: 100.0 },
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![port_add_a, port_add_b, port_add_out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_out,
        Node {
            kind: NodeKindKey::new("demo.output"),
            kind_version: 1,
            pos: CanvasPoint { x: 840.0, y: 140.0 },
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![port_out_in],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_weird,
        Node {
            kind: NodeKindKey::new(WEIRD_KIND),
            kind_version: 1,
            pos: CanvasPoint { x: 560.0, y: 300.0 },
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![
                port_weird_in_a,
                port_weird_in_b,
                port_weird_out_main,
                port_weird_out_aux,
            ],
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_value_a_out,
        Port {
            node: node_value_a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_value_b_out,
        Port {
            node: node_value_b,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_merge_in0,
        Port {
            node: node_merge,
            key: PortKey::new("in0"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_merge_in1,
        Port {
            node: node_merge,
            key: PortKey::new("in1"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_merge_out,
        Port {
            node: node_merge,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_add_a,
        Port {
            node: node_add,
            key: PortKey::new("a"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_add_b,
        Port {
            node: node_add,
            key: PortKey::new("b"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_add_out,
        Port {
            node: node_add,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_out_in,
        Port {
            node: node_out,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_weird_in_a,
        Port {
            node: node_weird,
            key: PortKey::new("in_a"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_weird_in_b,
        Port {
            node: node_weird,
            key: PortKey::new("in_b"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_weird_out_main,
        Port {
            node: node_weird,
            key: PortKey::new("out_main"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_weird_out_aux,
        Port {
            node: node_weird,
            key: PortKey::new("out_aux"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_value_a_out,
            to: port_merge_in0,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_merge_out,
            to: port_add_a,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_merge_out,
            to: port_weird_in_a,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_value_b_out,
            to: port_weird_in_b,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_weird_out_main,
            to: port_add_b,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_add_out,
            to: port_out_in,
        },
    );

    graph
}

struct NodeGraphDemoWindowState {
    ui: UiTree<App>,
    root: fret_core::NodeId,
}

#[derive(Debug, Default, Clone, Copy)]
struct DemoFloatPortalSpec;

impl PortalNumberEditSpec for DemoFloatPortalSpec {
    fn initial_value(&self, graph: &Graph, node: NodeId) -> Option<f64> {
        let node = graph.nodes.get(&node)?;
        if node.kind.0.as_str() != "demo.float" {
            return None;
        }
        Some(
            node.data
                .get("value")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        )
    }

    fn format_value(&self, value: f64) -> String {
        format!("{value:.3}")
    }

    fn submit_value(
        &self,
        graph: &Graph,
        node: NodeId,
        value: f64,
        _text: &str,
    ) -> PortalNumberEditSubmit {
        let from = graph
            .nodes
            .get(&node)
            .map(|n| n.data.clone())
            .unwrap_or(Value::Null);
        let to = set_float_value_in_node_data(from.clone(), value);
        let normalized = Some(format!("{value:.3}"));

        if from == to {
            return PortalNumberEditSubmit::Handled {
                normalized_text: normalized,
            };
        }

        PortalNumberEditSubmit::Commit {
            tx: GraphTransaction {
                label: Some("Set Float Value".to_string()),
                ops: vec![GraphOp::SetNodeData { id: node, from, to }],
            },
            normalized_text: normalized,
        }
    }

    fn supports_drag(&self, graph: &Graph, node: NodeId) -> bool {
        self.initial_value(graph, node).is_some()
    }

    fn drag_threshold_px(&self, graph: &Graph, node: NodeId) -> f32 {
        if self.initial_value(graph, node).is_none() {
            return 1.0;
        }
        2.0
    }

    fn drag_sensitivity_per_px(
        &self,
        graph: &Graph,
        node: NodeId,
        mode: fret_node::ui::PortalTextStepMode,
    ) -> Option<f64> {
        if self.initial_value(graph, node).is_none() {
            return None;
        }

        Some(match mode {
            fret_node::ui::PortalTextStepMode::Fine => 0.001,
            fret_node::ui::PortalTextStepMode::Normal => 0.01,
            fret_node::ui::PortalTextStepMode::Coarse => 0.1,
        })
    }

    fn step_size(
        &self,
        graph: &Graph,
        node: NodeId,
        mode: fret_node::ui::PortalTextStepMode,
    ) -> Option<f64> {
        if self.initial_value(graph, node).is_none() {
            return None;
        }

        Some(match mode {
            fret_node::ui::PortalTextStepMode::Fine => 0.025,
            fret_node::ui::PortalTextStepMode::Normal => 0.25,
            fret_node::ui::PortalTextStepMode::Coarse => 2.5,
        })
    }
}

#[derive(Default)]
struct NodeGraphDemoDriver;

impl NodeGraphDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> NodeGraphDemoWindowState {
        let models = app
            .global::<NodeGraphDemoModels>()
            .expect("NodeGraphDemoModels global must exist")
            .clone();
        let registry = app
            .global::<NodeRegistry>()
            .expect("NodeRegistry global must exist")
            .clone();
        let measured = app
            .global::<NodeGraphDemoMeasuredStores>()
            .expect("NodeGraphDemoMeasuredStores global must exist")
            .clone();
        let internals = app
            .global::<Arc<NodeGraphInternalsStore>>()
            .expect("NodeGraphInternalsStore global must exist")
            .clone();
        let internals_overlay = internals.clone();

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let graph = models.graph.clone();
        let view = models.view.clone();
        let edits = models.edits.clone();
        let overlays = models.overlays.clone();
        let group_rename_text = models.group_rename_text.clone();
        let style = NodeGraphStyle::from_theme(Theme::global(app));

        let presenter =
            MeasuredNodeGraphPresenter::new(DemoPresenter::new(registry), measured.manual.clone());
        let canvas = NodeGraphCanvas::new(graph.clone(), view)
            .with_presenter(presenter)
            .with_style(style.clone())
            .with_edit_queue(edits.clone())
            .with_overlay_state(overlays.clone())
            .with_internals_store(internals)
            .with_measured_output_store(measured.derived.clone())
            .with_close_command(CommandId::new("node_graph_demo.close"));
        let canvas_node = ui.create_node_retained(canvas);

        let overlay_host = NodeGraphOverlayHost::new(
            graph,
            edits,
            overlays,
            group_rename_text.clone(),
            canvas_node,
            style.clone(),
        );
        let overlay_node = ui.create_node_retained(overlay_host);
        let rename_input_node = ui.create_node_retained(BoundTextInput::new(group_rename_text));
        ui.set_children(overlay_node, vec![rename_input_node]);

        let portal_root = "node_graph_demo.portal";
        let portal_style = style.clone();
        let portal_editor = PortalNumberEditor::new(portal_root);
        let portal_graph_model = models.graph.clone();

        let portal = NodeGraphPortalHost::new(
            models.graph.clone(),
            models.view.clone(),
            measured.manual.clone(),
            style.clone(),
            portal_root,
            move |ecx: &mut fret_ui::elements::ElementContext<'_, App>,
                  graph: &Graph,
                  layout: NodeGraphPortalNodeLayout|
                  -> Vec<AnyElement> {
                let Some(node) = graph.nodes.get(&layout.node) else {
                    return Vec::new();
                };
                if node.kind.0.as_str() != "demo.float" {
                    return Vec::new();
                }

                portal_editor.render_number_input_for_node(
                    ecx,
                    portal_graph_model.clone(),
                    graph,
                    layout,
                    &portal_style,
                    layout.node,
                    &DemoFloatPortalSpec,
                )
            },
        )
        .with_edit_queue(models.edits.clone())
        .with_canvas_focus_target(canvas_node)
        .with_command_handler(PortalNumberEditHandler::new(
            portal_root,
            DemoFloatPortalSpec,
        ));
        let portal_node = ui.create_node_retained(portal);

        let controls =
            NodeGraphControlsOverlay::new(canvas_node, models.view.clone(), style.clone());
        let controls_node = ui.create_node_retained(controls);

        let tuning = NodeGraphTuningOverlay::new(canvas_node, models.view.clone(), style.clone());
        let tuning_node = ui.create_node_retained(tuning);

        let minimap = NodeGraphMiniMapOverlay::new(
            canvas_node,
            models.graph.clone(),
            models.view.clone(),
            internals_overlay,
            style.clone(),
        );
        let minimap_node = ui.create_node_retained(minimap);

        let root = ui.create_node_retained(NodeGraphEditor::new());
        ui.set_children(
            root,
            vec![
                canvas_node,
                portal_node,
                controls_node,
                tuning_node,
                minimap_node,
                overlay_node,
            ],
        );
        ui.set_root(root);

        NodeGraphDemoWindowState { ui, root }
    }
}

impl WinitAppDriver for NodeGraphDemoDriver {
    type WindowState = NodeGraphDemoWindowState;

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
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: fret_app::CommandId,
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

        if command.as_str() == "node_graph_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if command.as_str() == CMD_TOGGLE_WEIRD_LAYOUT {
            let Some(toggle) = app.global::<Arc<DemoWeirdLayoutMeasuredState>>().cloned() else {
                return;
            };
            let Some(measured) = app.global::<NodeGraphDemoMeasuredStores>().cloned() else {
                return;
            };
            let Some(models) = app.global::<NodeGraphDemoModels>().cloned() else {
                return;
            };

            let enabled = toggle.toggle();
            let pin_radius = 6.0;
            let weird_targets = models
                .graph
                .read_ref(app, |g| {
                    let mut targets = Vec::new();
                    for (node_id, node) in &g.nodes {
                        if node.kind.0.as_str() != WEIRD_KIND {
                            continue;
                        }

                        let mut ports = Vec::new();
                        for port_id in &node.ports {
                            let Some(port) = g.ports.get(port_id) else {
                                continue;
                            };
                            let key = port.key.0.as_str();
                            if matches!(key, "in_a" | "in_b" | "out_main" | "out_aux") {
                                ports.push((*port_id, key.to_string()));
                            }
                        }

                        targets.push((*node_id, ports));
                    }
                    targets
                })
                .ok();

            if let Some(targets) = weird_targets {
                let weird_size_px = if enabled {
                    (420.0, 120.0)
                } else {
                    (280.0, 240.0)
                };
                measured.manual.update(|node_sizes, anchors| {
                    for (node_id, ports) in targets {
                        if enabled {
                            node_sizes.insert(node_id, weird_size_px);
                            for (port_id, key) in ports {
                                if let Some(anchor) = DemoPresenter::weird_anchor_for_key(
                                    true,
                                    &key,
                                    weird_size_px,
                                    pin_radius,
                                ) {
                                    anchors.insert(port_id, anchor);
                                }
                            }
                        } else {
                            node_sizes.remove(&node_id);
                            for (port_id, _) in ports {
                                anchors.remove(&port_id);
                            }
                        }
                    }
                });
                app.request_redraw(window);
            }
        }

        if command.as_str() == CMD_LOG_INTERNALS {
            let Some(internals) = app.global::<Arc<NodeGraphInternalsStore>>().cloned() else {
                return;
            };
            let snap = internals.snapshot();
            tracing::info!(
                zoom = snap.transform.zoom,
                pan_x = snap.transform.pan.x,
                pan_y = snap.transform.pan.y,
                nodes = snap.nodes_window.len(),
                ports = snap.ports_window.len(),
                "node graph internals snapshot"
            );
        }

        if command.as_str() == CMD_LOG_MEASURED {
            let Some(measured) = app.global::<NodeGraphDemoMeasuredStores>().cloned() else {
                return;
            };
            tracing::info!(
                manual_rev = measured.manual.revision(),
                derived_rev = measured.derived.revision(),
                "node graph measured stores (manual vs derived)"
            );
            return;
        }

        if command.as_str() == CMD_BUMP_FLOAT_VALUE {
            let Some(models) = app.global::<NodeGraphDemoModels>().cloned() else {
                return;
            };

            let selected = models
                .view
                .read_ref(app, |s| s.selected_nodes.clone())
                .unwrap_or_default();
            if selected.is_empty() {
                tracing::info!("select a Float node first (demo.float)");
                return;
            }

            let ops = models
                .graph
                .read_ref(app, |g| {
                    let mut ops = Vec::new();
                    for node_id in &selected {
                        let Some(node) = g.nodes.get(node_id) else {
                            continue;
                        };
                        if node.kind.0.as_str() != "demo.float" {
                            continue;
                        }

                        let from = node.data.clone();
                        let mut obj = from.as_object().cloned().unwrap_or_default();
                        let cur = obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let next = cur + 0.1;
                        let Some(num) = serde_json::Number::from_f64(next) else {
                            continue;
                        };
                        obj.insert("value".to_string(), serde_json::Value::Number(num));
                        let to = serde_json::Value::Object(obj);

                        ops.push(GraphOp::SetNodeData {
                            id: *node_id,
                            from,
                            to,
                        });
                    }
                    ops
                })
                .unwrap_or_default();
            if ops.is_empty() {
                return;
            }

            let tx = GraphTransaction {
                label: Some("Bump Float Value".to_string()),
                ops,
            };
            let _ = models.edits.update(app, |q, _cx| q.push(tx));
            return;
        }
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

fn set_float_value_in_node_data(mut data: Value, value: f64) -> Value {
    match &mut data {
        Value::Object(map) => {
            map.insert("value".to_string(), Value::from(value));
            data
        }
        _ => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), Value::from(value));
            Value::Object(map)
        }
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
    register_node_graph_commands(app.commands_mut());
    register_demo_commands(app.commands_mut());
    install_default_keybindings_into_keymap(&mut app);

    let mut graph_value = build_demo_graph();
    // Concretize once at startup so dynamic ports (e.g. `fret.variadic_merge`) don't "surprise"
    // the user on the first unrelated edit.
    let _ = apply_transaction_with_profile(
        &mut graph_value,
        &mut DataflowProfile::new(),
        &GraphTransaction::new(),
    );

    let graph = app.models_mut().insert(graph_value);
    let view = app.models_mut().insert(NodeGraphViewState::default());
    let edits = app.models_mut().insert(NodeGraphEditQueue::default());
    let overlays = app.models_mut().insert(NodeGraphOverlayState::default());
    let group_rename_text = app.models_mut().insert(String::new());
    app.set_global(NodeGraphDemoModels {
        graph,
        view,
        edits,
        overlays,
        group_rename_text,
    });
    app.set_global(build_demo_registry());
    app.set_global(NodeGraphDemoMeasuredStores {
        manual: Arc::new(MeasuredGeometryStore::new()),
        derived: Arc::new(MeasuredGeometryStore::new()),
    });
    app.set_global(Arc::new(NodeGraphInternalsStore::new()));
    app.set_global(Arc::new(DemoWeirdLayoutMeasuredState::new()));

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo node_graph_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    run_app(config, app, NodeGraphDemoDriver::default()).map_err(anyhow::Error::from)
}

fn install_default_keybindings_into_keymap(app: &mut App) {
    let mut bindings: Vec<Binding> = Vec::new();

    for (id, meta) in app.commands().iter() {
        for kb in meta.default_keybindings.iter().cloned() {
            bindings.push(Binding {
                platform: kb.platform,
                sequence: vec![kb.chord],
                when: kb.when.clone().or_else(|| meta.when.clone()),
                command: Some(id.clone()),
            });
        }
    }

    if bindings.is_empty() {
        return;
    }

    app.with_global_mut(KeymapService::default, |svc, _app| {
        for b in bindings {
            svc.keymap.push_binding(b);
        }
    });
}

fn kb(platform: PlatformFilter, key: KeyCode, mods: Modifiers) -> DefaultKeybinding {
    DefaultKeybinding {
        platform,
        chord: KeyChord::new(key, mods),
        when: None,
    }
}

fn register_demo_commands(registry: &mut CommandRegistry) {
    let mac_cmd = |key: KeyCode| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        )
    };
    let win_ctrl = |key: KeyCode| {
        kb(
            PlatformFilter::Windows,
            key,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )
    };
    let linux_ctrl = |key: KeyCode| {
        kb(
            PlatformFilter::Linux,
            key,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )
    };
    let web_ctrl = |key: KeyCode| {
        kb(
            PlatformFilter::Web,
            key,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )
    };

    registry.register(
        CommandId::new(CMD_TOGGLE_WEIRD_LAYOUT),
        CommandMeta::new("Toggle Weird Layout")
            .with_category("Demo")
            .with_keywords(["toggle", "layout", "anchors", "geometry"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyL),
                win_ctrl(KeyCode::KeyL),
                linux_ctrl(KeyCode::KeyL),
                web_ctrl(KeyCode::KeyL),
            ]),
    );

    registry.register(
        CommandId::new(CMD_LOG_INTERNALS),
        CommandMeta::new("Log NodeGraph Internals")
            .with_category("Demo")
            .with_keywords(["internals", "handles", "bounds"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyI),
                win_ctrl(KeyCode::KeyI),
                linux_ctrl(KeyCode::KeyI),
                web_ctrl(KeyCode::KeyI),
            ]),
    );

    registry.register(
        CommandId::new(CMD_LOG_MEASURED),
        CommandMeta::new("Log NodeGraph Measured Stores")
            .with_category("Demo")
            .with_keywords(["measured", "handleBounds", "sizes"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyM),
                win_ctrl(KeyCode::KeyM),
                linux_ctrl(KeyCode::KeyM),
                web_ctrl(KeyCode::KeyM),
            ]),
    );

    registry.register(
        CommandId::new(CMD_BUMP_FLOAT_VALUE),
        CommandMeta::new("Bump Float Node Value")
            .with_category("Demo")
            .with_keywords(["float", "value", "edit", "transaction"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::ArrowUp),
                win_ctrl(KeyCode::ArrowUp),
                linux_ctrl(KeyCode::ArrowUp),
                web_ctrl(KeyCode::ArrowUp),
            ]),
    );
}
