use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use fret_app::App;
use fret_app::{CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, Modifiers,
    MouseButton, Point, Px, Rect, SceneOp, Size, TextBlobId, TextConstraints, TextOverflow,
    TextWrap,
};
use fret_launch::{
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_runtime::{
    CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, KeyChord, PlatformFilter,
    WhenExpr,
};
use fret_ui::Theme;
use fret_ui::retained_bridge::{BoundTextInput, UiTreeRetainedExt as _, *};
use fret_ui::{UiFrameCx, UiHost, UiTree};
use serde_json::Value;

use crate::node_graph_tuning_overlay::{NodeGraphTuningCommands, NodeGraphTuningOverlay};
use fret_app::install_command_default_keybindings_into_keymap;

use fret_node::Graph;
use fret_node::GraphId;
use fret_node::TypeDesc;
use fret_node::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Node, NodeId, NodeKindKey, Port};
use fret_node::core::{PortCapacity, PortDirection, PortId, PortKey, PortKind};
use fret_node::io::NodeGraphViewState;
use fret_node::io::NodeGraphViewStateFileV1;
use fret_node::ops::{GraphOp, GraphTransaction};
use fret_node::profile::{DataflowProfile, apply_transaction_with_profile};
use fret_node::rules::{InsertNodeTemplate, PortTemplate};
use fret_node::runtime::store::NodeGraphStore;
use fret_node::schema::{
    NodeKindMigrateError, NodeKindMigrator, NodeRegistry, NodeSchema, PortDecl,
};
use fret_node::ui::canvas::RejectNonFiniteTx;
use fret_node::ui::presenter::{
    EdgeMarker, EdgeRenderHint, EdgeRouteKind, InsertNodeCandidate, NodeGraphContextMenuItem,
    NodeGraphPresenter, PortAnchorHint,
};
use fret_node::ui::style::{NodeGraphBackgroundPattern, NodeGraphStyle};
use fret_node::ui::{
    MeasuredGeometryStore, MeasuredNodeGraphPresenter, NodeGraphA11yFocusedEdge,
    NodeGraphA11yFocusedNode, NodeGraphA11yFocusedPort, NodeGraphBlackboardOverlay,
    NodeGraphCanvas, NodeGraphControlsOverlay, NodeGraphDiagAnchor, NodeGraphDiagConnectingFlag,
    NodeGraphEdgeToolbar, NodeGraphEdgeTypes, NodeGraphEditQueue, NodeGraphEditor,
    NodeGraphInternalsStore, NodeGraphMiniMapOverlay, NodeGraphNodeToolbar, NodeGraphNodeTypes,
    NodeGraphOverlayHost, NodeGraphOverlayState, NodeGraphPanel, NodeGraphPanelPosition,
    NodeGraphPortalHost, NodeGraphPortalNodeLayout, NodeGraphPresetFamily, NodeGraphPresetSkinV1,
    NodeGraphToolbarAlign, NodeGraphToolbarPosition, PortalNumberEditHandler, PortalNumberEditSpec,
    PortalNumberEditSubmit, PortalNumberEditor, RegistryNodeGraphPresenter,
    register_node_graph_commands,
};

#[derive(Clone)]
struct NodeGraphDemoModels {
    store: fret_runtime::Model<NodeGraphStore>,
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

const CMD_TOGGLE_WEIRD_LAYOUT: &str = "node_graph_demo.toggle_weird_layout";
const CMD_LOG_INTERNALS: &str = "node_graph_demo.log_internals";
const CMD_LOG_MEASURED: &str = "node_graph_demo.log_measured";
const CMD_BUMP_FLOAT_VALUE: &str = "node_graph_demo.bump_float_value";
const CMD_CYCLE_BACKGROUND_PATTERN: &str = "node_graph_demo.cycle_background_pattern";
const CMD_CYCLE_PRESET_FAMILY: &str = "node_graph_demo.cycle_preset_family";
const CMD_TOGGLE_WIRE_GLOW: &str = "node_graph_demo.toggle_wire_glow";
const CMD_RESET_GRAPH: &str = "node_graph_demo.reset_graph";
const CMD_SPAWN_STRESS_1K: &str = "node_graph_demo.spawn_stress_1k";
const CMD_SPAWN_STRESS_5K: &str = "node_graph_demo.spawn_stress_5k";
const CMD_SPAWN_STRESS_10K: &str = "node_graph_demo.spawn_stress_10k";
const CMD_UPGRADE_GRAPH: &str = "node_graph_demo.upgrade_graph";
const CMD_TOGGLE_HELP_OVERLAY: &str = "node_graph_demo.toggle_help_overlay";
const CMD_TOGGLE_TOOLBARS: &str = "node_graph_demo.toggle_toolbars";
const CMD_TOGGLE_BLACKBOARD_OVERLAY: &str = "node_graph_demo.toggle_blackboard_overlay";
const CMD_TOGGLE_CONTROLS_PLACEMENT: &str = "node_graph_demo.toggle_controls_placement";
const CMD_TOGGLE_MINIMAP_PLACEMENT: &str = "node_graph_demo.toggle_minimap_placement";
const WEIRD_KIND: &str = "demo.weird_layout";

#[derive(Debug)]
struct NodeGraphDemoStyleState {
    background_pattern: AtomicUsize,
}

impl NodeGraphDemoStyleState {
    fn new() -> Self {
        Self {
            background_pattern: AtomicUsize::new(0),
        }
    }

    fn background_pattern(&self) -> NodeGraphBackgroundPattern {
        match self.background_pattern.load(Ordering::Relaxed) % 3 {
            1 => NodeGraphBackgroundPattern::Dots,
            2 => NodeGraphBackgroundPattern::Cross,
            _ => NodeGraphBackgroundPattern::Lines,
        }
    }

    fn cycle_background_pattern(&self) -> NodeGraphBackgroundPattern {
        let next = self
            .background_pattern
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        match next % 3 {
            1 => NodeGraphBackgroundPattern::Dots,
            2 => NodeGraphBackgroundPattern::Cross,
            _ => NodeGraphBackgroundPattern::Lines,
        }
    }
}

#[derive(Debug)]
struct NodeGraphDemoOverlayToggles {
    show_help: AtomicBool,
    show_toolbars: AtomicBool,
    show_blackboard: AtomicBool,
    controls_in_panel: AtomicBool,
    minimap_in_panel: AtomicBool,
}

impl NodeGraphDemoOverlayToggles {
    fn new() -> Self {
        Self {
            show_help: AtomicBool::new(true),
            show_toolbars: AtomicBool::new(true),
            show_blackboard: AtomicBool::new(true),
            controls_in_panel: AtomicBool::new(true),
            minimap_in_panel: AtomicBool::new(true),
        }
    }

    fn show_help(&self) -> bool {
        self.show_help.load(Ordering::Relaxed)
    }

    fn show_toolbars(&self) -> bool {
        self.show_toolbars.load(Ordering::Relaxed)
    }

    fn show_blackboard(&self) -> bool {
        self.show_blackboard.load(Ordering::Relaxed)
    }

    fn controls_in_panel(&self) -> bool {
        self.controls_in_panel.load(Ordering::Relaxed)
    }

    fn minimap_in_panel(&self) -> bool {
        self.minimap_in_panel.load(Ordering::Relaxed)
    }

    fn toggle_show_help(&self) {
        self.show_help.store(!self.show_help(), Ordering::Relaxed);
    }

    fn toggle_show_toolbars(&self) {
        self.show_toolbars
            .store(!self.show_toolbars(), Ordering::Relaxed);
    }

    fn toggle_show_blackboard(&self) {
        self.show_blackboard
            .store(!self.show_blackboard(), Ordering::Relaxed);
    }

    fn toggle_controls_placement(&self) {
        self.controls_in_panel
            .store(!self.controls_in_panel(), Ordering::Relaxed);
    }

    fn toggle_minimap_placement(&self) {
        self.minimap_in_panel
            .store(!self.minimap_in_panel(), Ordering::Relaxed);
    }
}

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
                hint.end_marker = Some(EdgeMarker::arrow(12.0));
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

    fn list_conversions(&mut self, graph: &Graph, a: PortId, b: PortId) -> Vec<InsertNodeTemplate> {
        let Some(port_a) = graph.ports.get(&a) else {
            return Vec::new();
        };
        let Some(port_b) = graph.ports.get(&b) else {
            return Vec::new();
        };

        let (from, to) = match (port_a.dir, port_b.dir) {
            (PortDirection::Out, PortDirection::In) => (port_a, port_b),
            (PortDirection::In, PortDirection::Out) => (port_b, port_a),
            _ => return Vec::new(),
        };

        let (Some(from_ty), Some(to_ty)) = (from.ty.as_ref(), to.ty.as_ref()) else {
            return Vec::new();
        };

        if !matches!((from_ty, to_ty), (TypeDesc::Float, TypeDesc::Int)) {
            return Vec::new();
        }

        let mk_template = |kind: &str| InsertNodeTemplate {
            kind: NodeKindKey::new(kind),
            kind_version: 1,
            collapsed: false,
            data: serde_json::Value::Null,
            ports: vec![
                PortTemplate {
                    key: PortKey::new("in"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
                PortTemplate {
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: Some(TypeDesc::Int),
                    data: serde_json::Value::Null,
                },
            ],
            input: PortKey::new("in"),
            output: PortKey::new("out"),
        };

        // Return multiple options to force the conversion picker (avoid auto-insert mutation in
        // diagnostics runs).
        vec![
            mk_template("demo.convert.float_to_int.cast"),
            mk_template("demo.convert.float_to_int.round"),
        ]
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

#[derive(Debug)]
struct DemoFloatMigrator;

impl NodeKindMigrator for DemoFloatMigrator {
    fn migrate(
        &self,
        from_version: u32,
        to_version: u32,
        data: &Value,
    ) -> Result<Value, NodeKindMigrateError> {
        if from_version == to_version {
            return Ok(data.clone());
        }
        if from_version != 0 || to_version != 1 {
            return Err(NodeKindMigrateError::message(format!(
                "unsupported float migration: {from_version} -> {to_version}"
            )));
        }

        let mut obj = match data {
            Value::Object(map) => map.clone(),
            Value::Number(n) => {
                let mut map = serde_json::Map::new();
                map.insert("val".to_string(), Value::Number(n.clone()));
                map
            }
            _ => serde_json::Map::new(),
        };

        let value = obj
            .get("value")
            .and_then(|v| v.as_f64())
            .or_else(|| obj.get("val").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);

        obj.insert("value".to_string(), Value::from(value));
        obj.remove("val");
        Ok(Value::Object(obj))
    }
}

fn build_demo_registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();

    reg.register(NodeSchema {
        kind: NodeKindKey::new("demo.float"),
        latest_kind_version: 1,
        kind_aliases: vec![NodeKindKey::new("demo.float.v0")],
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
    reg.register_migrator(NodeKindKey::new("demo.float"), Arc::new(DemoFloatMigrator));

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
        kind: NodeKindKey::new("demo.int_sink"),
        latest_kind_version: 1,
        kind_aliases: Vec::new(),
        title: "Int Sink".to_string(),
        category: vec!["Demo".to_string()],
        keywords: vec!["int".to_string(), "sink".to_string()],
        ports: vec![PortDecl {
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Int),
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

fn build_demo_graph(graph_id: GraphId) -> Graph {
    let mut graph = Graph::new(graph_id);

    let node_value_a = NodeId::new();
    let node_value_b = NodeId::new();
    let node_merge = NodeId::new();
    let node_add = NodeId::new();
    let node_out = NodeId::new();
    let node_weird = NodeId::new();
    let node_int_sink = NodeId::new();

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
    let port_int_sink_in = PortId::new();

    graph.nodes.insert(
        node_value_a,
        Node {
            kind: NodeKindKey::new("demo.float.v0"),
            kind_version: 0,
            pos: CanvasPoint { x: 40.0, y: 60.0 },
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
            ports: vec![port_value_a_out],
            data: serde_json::json!({ "val": 0.25 }),
        },
    );
    graph.nodes.insert(
        node_value_b,
        Node {
            kind: NodeKindKey::new("demo.float"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 170.0 },
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
            ports: vec![port_out_in],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_int_sink,
        Node {
            kind: NodeKindKey::new("demo.int_sink"),
            kind_version: 1,
            pos: CanvasPoint { x: 840.0, y: 320.0 },
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
            ports: vec![port_int_sink_in],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_weird,
        Node {
            kind: NodeKindKey::new(WEIRD_KIND),
            kind_version: 1,
            pos: CanvasPoint { x: 560.0, y: 300.0 },
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_int_sink_in,
        Port {
            node: node_int_sink,
            key: PortKey::new("in"),
            dir: PortDirection::In,
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
        port_weird_in_a,
        Port {
            node: node_weird,
            key: PortKey::new("in_a"),
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
    graph.ports.insert(
        port_weird_in_b,
        Port {
            node: node_weird,
            key: PortKey::new("in_b"),
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
    graph.ports.insert(
        port_weird_out_main,
        Port {
            node: node_weird,
            key: PortKey::new("out_main"),
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
        port_weird_out_aux,
        Port {
            node: node_weird,
            key: PortKey::new("out_aux"),
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

    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_value_a_out,
            to: port_merge_in0,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_merge_out,
            to: port_add_a,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_merge_out,
            to: port_weird_in_a,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_value_b_out,
            to: port_weird_in_b,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_weird_out_main,
            to: port_add_b,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_add_out,
            to: port_out_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    graph
}

fn build_stress_graph(graph_id: GraphId, target_nodes: usize) -> Graph {
    let mut graph = Graph::new(graph_id);

    // Build a mostly-regular, large graph intended for performance/conformance stress testing.
    //
    // Shape:
    // - A chain of `demo.add` nodes arranged in a grid.
    // - Each add node gets its `b` input from a nearby `demo.float`.
    // - The `a` input is chained from the previous add output, starting from a root float.
    //
    // This produces both many nodes and many short-ish edges without relying on dynamic ports.
    let add_nodes = target_nodes.saturating_sub(1) / 2;
    let float_nodes = add_nodes.saturating_add(1);

    let cols: usize = 64;
    let x_step = 360.0f32;
    let y_step = 220.0f32;

    let float_x_offset = -260.0f32;
    let float_y_offset = 40.0f32;

    let mut float_out_ports: Vec<PortId> = Vec::with_capacity(float_nodes);
    for i in 0..float_nodes {
        let node_id = NodeId::new();
        let port_out = PortId::new();

        let col = i % cols;
        let row = i / cols;
        let x = col as f32 * x_step + float_x_offset;
        let y = row as f32 * y_step + float_y_offset;
        let value = (i as f64) * 0.001;

        graph.nodes.insert(
            node_id,
            Node {
                kind: NodeKindKey::new("demo.float"),
                kind_version: 1,
                pos: CanvasPoint { x, y },
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
                ports: vec![port_out],
                data: serde_json::json!({ "value": value }),
            },
        );
        graph.ports.insert(
            port_out,
            Port {
                node: node_id,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: Some(TypeDesc::Float),
                data: serde_json::Value::Null,
            },
        );

        float_out_ports.push(port_out);
    }

    let mut prev_out: Option<PortId> = None;
    for i in 0..add_nodes {
        let node_id = NodeId::new();
        let port_a = PortId::new();
        let port_b = PortId::new();
        let port_out = PortId::new();

        let col = i % cols;
        let row = i / cols;
        let x = col as f32 * x_step;
        let y = row as f32 * y_step;

        graph.nodes.insert(
            node_id,
            Node {
                kind: NodeKindKey::new("demo.add"),
                kind_version: 1,
                pos: CanvasPoint { x, y },
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
                ports: vec![port_a, port_b, port_out],
                data: serde_json::Value::Null,
            },
        );
        graph.ports.insert(
            port_a,
            Port {
                node: node_id,
                key: PortKey::new("a"),
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
        graph.ports.insert(
            port_b,
            Port {
                node: node_id,
                key: PortKey::new("b"),
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
        graph.ports.insert(
            port_out,
            Port {
                node: node_id,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: Some(TypeDesc::Float),
                data: serde_json::Value::Null,
            },
        );

        let a_source = prev_out.unwrap_or(float_out_ports[0]);
        graph.edges.insert(
            EdgeId::new(),
            Edge {
                kind: EdgeKind::Data,
                from: a_source,
                to: port_a,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            EdgeId::new(),
            Edge {
                kind: EdgeKind::Data,
                from: float_out_ports[i + 1],
                to: port_b,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        prev_out = Some(port_out);
    }

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
struct NodeGraphDemoDriver {
    pending_view_state_save: bool,
    last_view_state_save_at: Option<Instant>,
}

impl NodeGraphDemoDriver {
    const VIEW_STATE_SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

    fn save_view_state(&mut self, app: &mut App) {
        let diagnostics_enabled = std::env::var("FRET_DIAG")
            .ok()
            .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0");
        if diagnostics_enabled {
            return;
        }

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
        let store = models.store.clone();
        let mut style = NodeGraphStyle::from_theme(Theme::global(app));
        let mut background = style.background_style();
        if let Some(style_state) = app.global::<Arc<NodeGraphDemoStyleState>>().cloned() {
            background.grid_pattern = style_state.background_pattern();
        }
        style = style.with_background_style(background);

        let presenter =
            MeasuredNodeGraphPresenter::new(DemoPresenter::new(registry), measured.manual.clone());

        // Stage 2 `edgeTypes`: demonstrate custom edge paths (used for paint + hit-testing).
        let edge_types = NodeGraphEdgeTypes::new().register_path(
            fret_node::ui::EdgeTypeKey::new("data"),
            |_graph, edge_id, _style, _hint, input| {
                let zoom = input.zoom.max(1.0e-6);

                let dx = input.to.x.0 - input.from.x.0;
                let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
                let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
                let bend = 48.0 / zoom;

                let c1 = Point::new(Px(input.from.x.0 + dir * ctrl), Px(input.from.y.0 - bend));
                let c2 = Point::new(Px(input.to.x.0 - dir * ctrl), Px(input.to.y.0 - bend));

                let q = |v: f32, step: f32| -> i64 {
                    if !v.is_finite() {
                        return 0;
                    }
                    (v / step).round() as i64
                };

                let mut hasher = DefaultHasher::new();
                edge_id.hash(&mut hasher);
                q(input.from.x.0, 0.01).hash(&mut hasher);
                q(input.from.y.0, 0.01).hash(&mut hasher);
                q(input.to.x.0, 0.01).hash(&mut hasher);
                q(input.to.y.0, 0.01).hash(&mut hasher);
                q(zoom, 0.0001).hash(&mut hasher);

                Some(fret_node::ui::edge_types::EdgeCustomPath {
                    cache_key: hasher.finish(),
                    commands: vec![
                        fret_core::PathCommand::MoveTo(input.from),
                        fret_core::PathCommand::CubicTo {
                            ctrl1: c1,
                            ctrl2: c2,
                            to: input.to,
                        },
                    ],
                })
            },
        );

        let preset_skin = app
            .global::<Arc<NodeGraphPresetSkinV1>>()
            .cloned()
            .unwrap_or_else(|| {
                NodeGraphPresetSkinV1::new_from_snapshot(
                    Theme::global(app).snapshot(),
                    NodeGraphPresetFamily::WorkflowClean,
                )
            });

        let diagnostics_enabled = std::env::var("FRET_DIAG")
            .ok()
            .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0");
        let diag_anchor_ports: Vec<PortId> = if diagnostics_enabled {
            graph
                .read_ref(app, |g| {
                    let find_port =
                        |node_kind: &str, port_key: &str, dir: PortDirection| -> Option<PortId> {
                            g.ports.iter().find_map(|(id, port)| {
                                if port.dir != dir || port.key.0.as_str() != port_key {
                                    return None;
                                }
                                let node = g.nodes.get(&port.node)?;
                                if node.kind.0.as_str() != node_kind {
                                    return None;
                                }
                                Some(*id)
                            })
                        };

                    let float_out = find_port("demo.float.v0", "out", PortDirection::Out)?;
                    let float_in = find_port("fret.variadic_merge", "in1", PortDirection::In)?;
                    let float_out_b = find_port("demo.float", "out", PortDirection::Out)?;
                    let int_in = find_port("demo.int_sink", "in", PortDirection::In)?;
                    Some(vec![float_out, float_in, float_out_b, int_in])
                })
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view)
            .with_store(store.clone())
            .with_middleware(RejectNonFiniteTx)
            .with_presenter(presenter)
            .with_edge_types(edge_types)
            .with_style(style.clone())
            .with_skin(preset_skin)
            .with_edit_queue(edits.clone())
            .with_overlay_state(overlays.clone())
            .with_internals_store(internals)
            .with_measured_output_store(measured.derived.clone())
            .with_close_command(CommandId::new("node_graph_demo.close"));
        if !diag_anchor_ports.is_empty() {
            canvas = canvas.with_diagnostics_anchor_ports(4, diag_anchor_ports.clone());
        }
        let canvas_node = ui.create_node_retained(canvas);

        let a11y_port =
            ui.create_node_retained(NodeGraphA11yFocusedPort::new(internals_overlay.clone()));
        let a11y_edge =
            ui.create_node_retained(NodeGraphA11yFocusedEdge::new(internals_overlay.clone()));
        let a11y_node =
            ui.create_node_retained(NodeGraphA11yFocusedNode::new(internals_overlay.clone()));
        let mut canvas_children = vec![a11y_port, a11y_edge, a11y_node];
        if diagnostics_enabled && diag_anchor_ports.len() == 4 {
            canvas_children.push(ui.create_node_retained(NodeGraphDiagConnectingFlag::new(
                internals_overlay.clone(),
                "node_graph_demo.flag.connecting",
            )));
            canvas_children.push(ui.create_node_retained(NodeGraphDiagAnchor::new(
                "node_graph_demo.anchor.float_out",
            )));
            canvas_children.push(
                ui.create_node_retained(NodeGraphDiagAnchor::new(
                    "node_graph_demo.anchor.float_in",
                )),
            );
            canvas_children.push(ui.create_node_retained(NodeGraphDiagAnchor::new(
                "node_graph_demo.anchor.float_out_b",
            )));
            canvas_children.push(
                ui.create_node_retained(NodeGraphDiagAnchor::new("node_graph_demo.anchor.int_in")),
            );
        }
        ui.set_children(canvas_node, canvas_children);

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

        let node_types = NodeGraphNodeTypes::new().register(
            NodeKindKey::new("demo.float"),
            move |ecx: &mut fret_ui::elements::ElementContext<'_, App>,
                  graph: &Graph,
                  layout: NodeGraphPortalNodeLayout| {
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
        );

        let portal = NodeGraphPortalHost::new(
            models.graph.clone(),
            models.view.clone(),
            measured.manual.clone(),
            style.clone(),
            portal_root,
            node_types.into_portal_renderer(),
        )
        .with_cull_margin_px(style.render_cull_margin_px)
        .with_edit_queue(models.edits.clone())
        .with_canvas_focus_target(canvas_node)
        .with_command_handler(PortalNumberEditHandler::new(
            portal_root,
            DemoFloatPortalSpec,
        ));
        let portal_node = ui.create_node_retained(portal);

        let toggles = app
            .global::<Arc<NodeGraphDemoOverlayToggles>>()
            .cloned()
            .unwrap_or_else(|| Arc::new(NodeGraphDemoOverlayToggles::new()));

        let controls_node = if toggles.controls_in_panel() {
            let controls_overlay =
                NodeGraphControlsOverlay::new(canvas_node, models.view.clone(), style.clone())
                    .in_panel_bounds();
            let controls_overlay_node = ui.create_node_retained(controls_overlay);

            let controls_panel = NodeGraphPanel::new(NodeGraphPanelPosition::TopRight)
                .with_margin_px(style.controls_margin);
            let controls_node = ui.create_node_retained(controls_panel);
            ui.set_children(controls_node, vec![controls_overlay_node]);
            Some(controls_node)
        } else {
            let controls_overlay =
                NodeGraphControlsOverlay::new(canvas_node, models.view.clone(), style.clone());
            Some(ui.create_node_retained(controls_overlay))
        };

        let tuning = NodeGraphTuningOverlay::new(canvas_node, models.view.clone(), style.clone())
            .with_store(store.clone())
            .with_commands(NodeGraphTuningCommands {
                reset_graph: CommandId::new(CMD_RESET_GRAPH),
                spawn_stress_1k: CommandId::new(CMD_SPAWN_STRESS_1K),
                spawn_stress_5k: CommandId::new(CMD_SPAWN_STRESS_5K),
                spawn_stress_10k: CommandId::new(CMD_SPAWN_STRESS_10K),
            });
        let tuning_node = ui.create_node_retained(tuning);

        let help_node = if toggles.show_help() {
            Some(ui.create_node_retained(DemoHelpOverlay::new(style.clone(), toggles.clone())))
        } else {
            None
        };

        let blackboard_node = if toggles.show_blackboard() {
            Some(ui.create_node_retained(NodeGraphBlackboardOverlay::new(
                models.graph.clone(),
                models.view.clone(),
                models.edits.clone(),
                models.overlays.clone(),
                canvas_node,
                style.clone(),
            )))
        } else {
            None
        };

        let minimap_node = if toggles.minimap_in_panel() {
            let minimap_overlay = NodeGraphMiniMapOverlay::new(
                canvas_node,
                models.graph.clone(),
                models.view.clone(),
                internals_overlay.clone(),
                style.clone(),
            )
            .with_store(store)
            .in_panel_bounds();
            let minimap_overlay_node = ui.create_node_retained(minimap_overlay);

            let minimap_panel = NodeGraphPanel::new(NodeGraphPanelPosition::BottomRight)
                .with_margin_px(style.minimap_margin);
            let minimap_node = ui.create_node_retained(minimap_panel);
            ui.set_children(minimap_node, vec![minimap_overlay_node]);
            Some(minimap_node)
        } else {
            let minimap_overlay = NodeGraphMiniMapOverlay::new(
                canvas_node,
                models.graph.clone(),
                models.view.clone(),
                internals_overlay.clone(),
                style.clone(),
            )
            .with_store(store);
            Some(ui.create_node_retained(minimap_overlay))
        };

        let (node_toolbar_node, edge_toolbar_node) = if toggles.show_toolbars() {
            let node_toolbar = NodeGraphNodeToolbar::new(
                canvas_node,
                models.graph.clone(),
                models.view.clone(),
                internals_overlay.clone(),
            )
            .with_position(NodeGraphToolbarPosition::Top)
            .with_align(NodeGraphToolbarAlign::Center)
            .with_gap_px(10.0);
            let node_toolbar_node = ui.create_node_retained(node_toolbar);
            let node_toolbar_content =
                ui.create_node_retained(DemoToolbarStrip::node_toolbar(canvas_node, style.clone()));
            ui.set_children(node_toolbar_node, vec![node_toolbar_content]);

            let edge_toolbar = NodeGraphEdgeToolbar::new(
                canvas_node,
                models.graph.clone(),
                models.view.clone(),
                internals_overlay.clone(),
            )
            .with_align_x(NodeGraphToolbarAlign::Center)
            .with_align_y(NodeGraphToolbarAlign::End)
            .with_offset_px(0.0, -10.0);
            let edge_toolbar_node = ui.create_node_retained(edge_toolbar);
            let edge_toolbar_content =
                ui.create_node_retained(DemoToolbarStrip::edge_toolbar(canvas_node, style.clone()));
            ui.set_children(edge_toolbar_node, vec![edge_toolbar_content]);

            (Some(node_toolbar_node), Some(edge_toolbar_node))
        } else {
            (None, None)
        };

        let root = ui.create_node_retained(NodeGraphEditor::new());
        let mut children: Vec<fret_core::NodeId> = vec![canvas_node, portal_node, tuning_node];
        if let Some(n) = controls_node {
            children.push(n);
        }
        if let Some(n) = minimap_node {
            children.push(n);
        }
        if let Some(n) = node_toolbar_node {
            children.push(n);
        }
        if let Some(n) = edge_toolbar_node {
            children.push(n);
        }
        if let Some(n) = help_node {
            children.push(n);
        }
        if let Some(n) = blackboard_node {
            children.push(n);
        }
        // Overlay host (TextInput-backed rename UI) should be last so it can appear above other
        // editor overlays when active.
        children.push(overlay_node);
        ui.set_children(root, children);
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
        let WinitWindowContext {
            app, state, window, ..
        } = context;

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.record_model_changes(window, changed);
        });
        state.ui.propagate_model_changes(app, changed);

        let Some(models) = app.global::<NodeGraphDemoModels>() else {
            return;
        };
        if changed.contains(&models.view.id()) {
            self.pending_view_state_save = true;
        }
        if self.pending_view_state_save {
            let now = Instant::now();
            let due = self.last_view_state_save_at.map_or(true, |t| {
                now.duration_since(t) >= Self::VIEW_STATE_SAVE_DEBOUNCE
            });
            if due {
                self.pending_view_state_save = false;
                self.last_view_state_save_at = Some(now);
                self.save_view_state(app);
            }
        }
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        let WinitWindowContext {
            app, state, window, ..
        } = context;
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.record_global_changes(app, window, changed);
        });
        state.ui.propagate_global_changes(app, changed);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        let consumed = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            if !svc.is_enabled() {
                return false;
            }
            if svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event) {
                return true;
            }
            svc.maybe_intercept_event_for_picking(app, window, event)
        });
        if consumed {
            return;
        }

        if matches!(event, Event::WindowCloseRequested) {
            self.save_view_state(app);
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
            self.save_view_state(app);
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if command.as_str() == CMD_UPGRADE_GRAPH {
            let Some(models) = app.global::<NodeGraphDemoModels>().cloned() else {
                return;
            };
            let Some(registry) = app.global::<NodeRegistry>().cloned() else {
                return;
            };

            let result = models.store.update(app, move |store, _cx| {
                let graph = store.graph();

                let canonicalize = registry.plan_canonicalize_kinds(graph);
                let mut migrate = registry.plan_migrate_nodes(graph);
                migrate.tx.label = Some("Upgrade Node Graph".to_string());

                let report = migrate.report;
                let rewrite_count = canonicalize.rewrites.len();

                if migrate.tx.is_empty() {
                    return (rewrite_count, report, false);
                }

                let ok = store.dispatch_transaction(&migrate.tx).is_ok();
                (rewrite_count, report, ok)
            });

            match result {
                Ok((rewrites, report, did_apply)) => {
                    if !did_apply && rewrites == 0 {
                        tracing::info!("upgrade: no changes required");
                    } else {
                        tracing::info!(
                            rewrites,
                            upgraded = report.upgraded.len(),
                            missing_schema = report.missing_schema.len(),
                            missing_migrator = report.missing_migrator.len(),
                            newer_than_schema = report.newer_than_schema.len(),
                            errors = report.errors.len(),
                            did_apply,
                            "upgrade: completed"
                        );
                    }
                }
                Err(_) => tracing::warn!("upgrade: store unavailable"),
            }

            app.request_redraw(window);
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

        if command.as_str() == CMD_CYCLE_BACKGROUND_PATTERN {
            let Some(style_state) = app.global::<Arc<NodeGraphDemoStyleState>>().cloned() else {
                return;
            };

            let pattern = style_state.cycle_background_pattern();
            tracing::info!(?pattern, "node graph demo background pattern changed");

            *state = Self::build_ui(app, window);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_CYCLE_PRESET_FAMILY {
            let Some(skin) = app.global::<Arc<NodeGraphPresetSkinV1>>().cloned() else {
                return;
            };

            let next = skin.cycle();
            tracing::info!(
                preset = next.display_name(),
                "node graph demo preset changed"
            );

            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_TOGGLE_WIRE_GLOW {
            let Some(skin) = app.global::<Arc<NodeGraphPresetSkinV1>>().cloned() else {
                return;
            };

            let enabled = skin.toggle_wire_glow();
            tracing::info!(enabled, "node graph demo wire glow toggled");

            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_TOGGLE_HELP_OVERLAY {
            let Some(toggles) = app.global::<Arc<NodeGraphDemoOverlayToggles>>().cloned() else {
                return;
            };
            toggles.toggle_show_help();
            *state = Self::build_ui(app, window);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_TOGGLE_TOOLBARS {
            let Some(toggles) = app.global::<Arc<NodeGraphDemoOverlayToggles>>().cloned() else {
                return;
            };
            toggles.toggle_show_toolbars();
            *state = Self::build_ui(app, window);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_TOGGLE_BLACKBOARD_OVERLAY {
            let Some(toggles) = app.global::<Arc<NodeGraphDemoOverlayToggles>>().cloned() else {
                return;
            };
            toggles.toggle_show_blackboard();
            *state = Self::build_ui(app, window);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_TOGGLE_CONTROLS_PLACEMENT {
            let Some(toggles) = app.global::<Arc<NodeGraphDemoOverlayToggles>>().cloned() else {
                return;
            };
            toggles.toggle_controls_placement();
            *state = Self::build_ui(app, window);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_TOGGLE_MINIMAP_PLACEMENT {
            let Some(toggles) = app.global::<Arc<NodeGraphDemoOverlayToggles>>().cloned() else {
                return;
            };
            toggles.toggle_minimap_placement();
            *state = Self::build_ui(app, window);
            app.request_redraw(window);
            return;
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

        if matches!(
            command.as_str(),
            CMD_RESET_GRAPH | CMD_SPAWN_STRESS_1K | CMD_SPAWN_STRESS_5K | CMD_SPAWN_STRESS_10K
        ) {
            let Some(models) = app.global::<NodeGraphDemoModels>().cloned() else {
                return;
            };
            let Some(measured) = app.global::<NodeGraphDemoMeasuredStores>().cloned() else {
                return;
            };
            let Some(persist) = app.global::<NodeGraphDemoViewStatePersistence>().cloned() else {
                return;
            };

            let graph_id = persist.graph_id;
            let next_graph = match command.as_str() {
                CMD_RESET_GRAPH => build_demo_graph(graph_id),
                CMD_SPAWN_STRESS_1K => build_stress_graph(graph_id, 1_000),
                CMD_SPAWN_STRESS_5K => build_stress_graph(graph_id, 5_000),
                CMD_SPAWN_STRESS_10K => build_stress_graph(graph_id, 10_000),
                _ => return,
            };

            measured.manual.update(|node_sizes, anchors| {
                node_sizes.clear();
                anchors.clear();
            });
            measured.derived.update(|node_sizes, anchors| {
                node_sizes.clear();
                anchors.clear();
            });

            let _ = models
                .edits
                .update(app, |q, _cx| *q = NodeGraphEditQueue::default());
            let _ = models
                .overlays
                .update(app, |o, _cx| *o = NodeGraphOverlayState::default());

            let mut next_view = NodeGraphViewState::default();
            next_view.sanitize_for_graph(&next_graph);

            let _ = models.store.update(app, |store, _cx| {
                store.replace_graph(next_graph);
                store.replace_view_state(next_view);
                store.clear_history();
            });

            app.request_redraw(window);
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
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        scene.clear();

        let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot_arc();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                Some(&mut state.ui),
                semantics_snapshot.as_deref(),
            )
        });

        if drive.request_redraw {
            app.request_redraw(window);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }

        if injected_any {
            let mut deferred_effects: Vec<Effect> = Vec::new();
            loop {
                let effects = app.flush_effects();
                if effects.is_empty() {
                    break;
                }

                let mut applied_any_command = false;
                for effect in effects {
                    match effect {
                        Effect::Command { window: w, command } => {
                            if w.is_none() || w == Some(window) {
                                let _ = state.ui.dispatch_command(app, services, &command);
                                applied_any_command = true;
                            } else {
                                deferred_effects.push(Effect::Command { window: w, command });
                            }
                        }
                        other => deferred_effects.push(other),
                    }
                }

                if !applied_any_command {
                    break;
                }
            }
            for effect in deferred_effects {
                app.push_effect(effect);
            }

            state.ui.request_semantics_snapshot();
            let mut frame =
                UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &mut state.ui,
                element_runtime,
                scene,
            );
            let _ = svc.maybe_dump_if_triggered();
            if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
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
    install_command_default_keybindings_into_keymap(&mut app);

    let graph_id = GraphId::from_u128(0x1350_0000_0000_0000_0000_0000_0000_00A1);
    let mut graph_value = build_demo_graph(graph_id);
    // Concretize once at startup so dynamic ports (e.g. `fret.variadic_merge`) don't "surprise"
    // the user on the first unrelated edit.
    let _ = apply_transaction_with_profile(
        &mut graph_value,
        &mut DataflowProfile::new(),
        &GraphTransaction::new(),
    );

    let view_state_path = fret_node::io::default_project_view_state_path(graph_value.graph_id);
    let diagnostics_enabled = std::env::var("FRET_DIAG")
        .ok()
        .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0");
    let mut view_value = if diagnostics_enabled {
        NodeGraphViewState::default()
    } else {
        match NodeGraphViewStateFileV1::load_json_if_exists(&view_state_path, graph_value.graph_id)
        {
            Ok(Some(file)) => file.state,
            Ok(None) => NodeGraphViewState::default(),
            Err(err) => {
                tracing::warn!(?err, "failed to load node graph view state; using defaults");
                NodeGraphViewState::default()
            }
        }
    };
    view_value.sanitize_for_graph(&graph_value);

    let store_value =
        NodeGraphStore::with_profile(graph_value, view_value, Box::new(DataflowProfile::new()));
    let graph = app.models_mut().insert(store_value.graph().clone());
    let view = app.models_mut().insert(store_value.view_state().clone());
    let store = app.models_mut().insert(store_value);
    let edits = app.models_mut().insert(NodeGraphEditQueue::default());
    let overlays = app.models_mut().insert(NodeGraphOverlayState::default());
    let group_rename_text = app.models_mut().insert(String::new());
    app.set_global(NodeGraphDemoModels {
        store,
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
    app.set_global(build_demo_registry());
    app.set_global(NodeGraphDemoMeasuredStores {
        manual: Arc::new(MeasuredGeometryStore::new()),
        derived: Arc::new(MeasuredGeometryStore::new()),
    });
    app.set_global(Arc::new(NodeGraphInternalsStore::new()));
    app.set_global(Arc::new(DemoWeirdLayoutMeasuredState::new()));
    app.set_global(Arc::new(NodeGraphDemoStyleState::new()));
    app.set_global(Arc::new(NodeGraphDemoOverlayToggles::new()));
    app.set_global(NodeGraphPresetSkinV1::new_from_snapshot(
        Theme::global(&app).snapshot(),
        NodeGraphPresetFamily::WorkflowClean,
    ));

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo node_graph_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    fret::run_native_demo(config, app, NodeGraphDemoDriver::default()).map_err(anyhow::Error::from)
}

fn kb(platform: PlatformFilter, key: KeyCode, mods: Modifiers) -> DefaultKeybinding {
    DefaultKeybinding {
        platform,
        sequence: vec![KeyChord::new(key, mods)],
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
    let mac_cmd_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                meta: true,
                shift: true,
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
    let win_ctrl_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Windows,
            key,
            Modifiers {
                ctrl: true,
                shift: true,
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
    let linux_ctrl_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Linux,
            key,
            Modifiers {
                ctrl: true,
                shift: true,
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
    let web_ctrl_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Web,
            key,
            Modifiers {
                ctrl: true,
                shift: true,
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

    registry.register(
        CommandId::new(CMD_CYCLE_BACKGROUND_PATTERN),
        CommandMeta::new("Cycle NodeGraph Background Pattern")
            .with_category("Demo")
            .with_keywords(["background", "grid", "pattern", "dots", "cross"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyB),
                win_ctrl(KeyCode::KeyB),
                linux_ctrl(KeyCode::KeyB),
                web_ctrl(KeyCode::KeyB),
            ]),
    );

    registry.register(
        CommandId::new(CMD_CYCLE_PRESET_FAMILY),
        CommandMeta::new("Cycle NodeGraph Preset Family")
            .with_category("Demo")
            .with_keywords(["preset", "theme", "skin", "style"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd_shift(KeyCode::KeyP),
                win_ctrl_shift(KeyCode::KeyP),
                linux_ctrl_shift(KeyCode::KeyP),
                web_ctrl_shift(KeyCode::KeyP),
            ]),
    );

    registry.register(
        CommandId::new(CMD_TOGGLE_WIRE_GLOW),
        CommandMeta::new("Toggle NodeGraph Wire Glow")
            .with_category("Demo")
            .with_keywords(["wire", "glow", "effect", "shadow"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd_shift(KeyCode::KeyG),
                win_ctrl_shift(KeyCode::KeyG),
                linux_ctrl_shift(KeyCode::KeyG),
                web_ctrl_shift(KeyCode::KeyG),
            ]),
    );

    registry.register(
        CommandId::new(CMD_TOGGLE_HELP_OVERLAY),
        CommandMeta::new("Toggle Demo Help Overlay")
            .with_category("Demo")
            .with_keywords(["help", "overlay", "shortcuts"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyH),
                win_ctrl(KeyCode::KeyH),
                linux_ctrl(KeyCode::KeyH),
                web_ctrl(KeyCode::KeyH),
            ]),
    );

    registry.register(
        CommandId::new(CMD_TOGGLE_TOOLBARS),
        CommandMeta::new("Toggle Node/Edge Toolbars")
            .with_category("Demo")
            .with_keywords(["toolbar", "overlay", "node", "edge"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyT),
                win_ctrl(KeyCode::KeyT),
                linux_ctrl(KeyCode::KeyT),
                web_ctrl(KeyCode::KeyT),
            ]),
    );

    registry.register(
        CommandId::new(CMD_TOGGLE_BLACKBOARD_OVERLAY),
        CommandMeta::new("Toggle Blackboard Overlay (Symbols)")
            .with_category("Demo")
            .with_keywords(["blackboard", "symbols", "variables", "overlay"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                kb(
                    PlatformFilter::Macos,
                    KeyCode::KeyB,
                    Modifiers {
                        meta: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Windows,
                    KeyCode::KeyB,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Linux,
                    KeyCode::KeyB,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Web,
                    KeyCode::KeyB,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
            ]),
    );

    registry.register(
        CommandId::new(CMD_TOGGLE_CONTROLS_PLACEMENT),
        CommandMeta::new("Toggle Controls Placement (Panel vs Floating)")
            .with_category("Demo")
            .with_keywords(["controls", "panel", "overlay", "placement"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                kb(
                    PlatformFilter::Macos,
                    KeyCode::KeyC,
                    Modifiers {
                        meta: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Windows,
                    KeyCode::KeyC,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Linux,
                    KeyCode::KeyC,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Web,
                    KeyCode::KeyC,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
            ]),
    );

    registry.register(
        CommandId::new(CMD_TOGGLE_MINIMAP_PLACEMENT),
        CommandMeta::new("Toggle MiniMap Placement (Panel vs Floating)")
            .with_category("Demo")
            .with_keywords(["minimap", "panel", "overlay", "placement"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                kb(
                    PlatformFilter::Macos,
                    KeyCode::KeyM,
                    Modifiers {
                        meta: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Windows,
                    KeyCode::KeyM,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Linux,
                    KeyCode::KeyM,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
                kb(
                    PlatformFilter::Web,
                    KeyCode::KeyM,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
            ]),
    );

    registry.register(
        CommandId::new(CMD_RESET_GRAPH),
        CommandMeta::new("Reset Demo Graph")
            .with_category("Demo")
            .with_keywords(["reset", "graph", "demo"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr")),
    );

    registry.register(
        CommandId::new(CMD_UPGRADE_GRAPH),
        CommandMeta::new("Upgrade Node Graph (Canonicalize + Migrate)")
            .with_category("Demo")
            .with_keywords(["upgrade", "migrate", "canonicalize", "schema", "version"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr"))
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyU),
                win_ctrl(KeyCode::KeyU),
                linux_ctrl(KeyCode::KeyU),
                web_ctrl(KeyCode::KeyU),
            ]),
    );

    registry.register(
        CommandId::new(CMD_SPAWN_STRESS_1K),
        CommandMeta::new("Spawn Stress Graph (1k nodes)")
            .with_category("Demo")
            .with_keywords(["stress", "graph", "perf", "1k"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr")),
    );
    registry.register(
        CommandId::new(CMD_SPAWN_STRESS_5K),
        CommandMeta::new("Spawn Stress Graph (5k nodes)")
            .with_category("Demo")
            .with_keywords(["stress", "graph", "perf", "5k"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr")),
    );
    registry.register(
        CommandId::new(CMD_SPAWN_STRESS_10K),
        CommandMeta::new("Spawn Stress Graph (10k nodes)")
            .with_category("Demo")
            .with_keywords(["stress", "graph", "perf", "10k"])
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("!focus.is_text_input").expect("valid when expr")),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoToolbarButton {
    Delete,
    Fit,
}

struct DemoToolbarLayout {
    panel: Rect,
    delete: Rect,
    fit: Rect,
}

struct DemoToolbarStrip {
    canvas_node: fret_core::NodeId,
    style: NodeGraphStyle,
    label: &'static str,
    hovered: Option<DemoToolbarButton>,
    pressed: Option<DemoToolbarButton>,
    text_blobs: Vec<TextBlobId>,
}

impl DemoToolbarStrip {
    const PAD_PX: f32 = 6.0;
    const GAP_PX: f32 = 6.0;
    const BUTTON_W_PX: f32 = 44.0;
    const BUTTON_H_PX: f32 = 22.0;
    const LABEL_W_PX: f32 = 92.0;

    fn node_toolbar(canvas_node: fret_core::NodeId, style: NodeGraphStyle) -> Self {
        Self {
            canvas_node,
            style,
            label: "Node Toolbar",
            hovered: None,
            pressed: None,
            text_blobs: Vec::new(),
        }
    }

    fn edge_toolbar(canvas_node: fret_core::NodeId, style: NodeGraphStyle) -> Self {
        Self {
            canvas_node,
            style,
            label: "Edge Toolbar",
            hovered: None,
            pressed: None,
            text_blobs: Vec::new(),
        }
    }

    fn panel_size_px(&self) -> (f32, f32) {
        let w = 2.0 * Self::PAD_PX
            + Self::LABEL_W_PX
            + Self::GAP_PX
            + 2.0 * Self::BUTTON_W_PX
            + Self::GAP_PX;
        let h = 2.0 * Self::PAD_PX + Self::BUTTON_H_PX;
        (w, h)
    }

    fn compute_layout(&self, bounds: Rect) -> DemoToolbarLayout {
        let (panel_w, panel_h) = self.panel_size_px();
        let panel = Rect::new(bounds.origin, Size::new(Px(panel_w), Px(panel_h)));

        let delete = Rect::new(
            Point::new(
                Px(panel.origin.x.0 + panel.size.width.0
                    - Self::PAD_PX
                    - 2.0 * Self::BUTTON_W_PX
                    - Self::GAP_PX),
                Px(panel.origin.y.0 + Self::PAD_PX),
            ),
            Size::new(Px(Self::BUTTON_W_PX), Px(Self::BUTTON_H_PX)),
        );
        let fit = Rect::new(
            Point::new(
                Px(panel.origin.x.0 + panel.size.width.0 - Self::PAD_PX - Self::BUTTON_W_PX),
                Px(panel.origin.y.0 + Self::PAD_PX),
            ),
            Size::new(Px(Self::BUTTON_W_PX), Px(Self::BUTTON_H_PX)),
        );

        DemoToolbarLayout { panel, delete, fit }
    }

    fn button_at(&self, bounds: Rect, position: Point) -> Option<DemoToolbarButton> {
        let layout = self.compute_layout(bounds);
        if layout.delete.contains(position) {
            return Some(DemoToolbarButton::Delete);
        }
        if layout.fit.contains(position) {
            return Some(DemoToolbarButton::Fit);
        }
        None
    }

    fn dispatch_button<H: UiHost>(&self, cx: &mut EventCx<'_, H>, btn: DemoToolbarButton) {
        cx.request_focus(self.canvas_node);
        let id = match btn {
            DemoToolbarButton::Delete => {
                CommandId::new(fret_node::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
            }
            DemoToolbarButton::Fit => {
                CommandId::new(fret_node::ui::commands::CMD_NODE_GRAPH_FRAME_SELECTION)
            }
        };
        cx.dispatch_command(id);
        cx.request_redraw();
    }

    fn label_for(btn: DemoToolbarButton) -> &'static str {
        match btn {
            DemoToolbarButton::Delete => "Del",
            DemoToolbarButton::Fit => "Fit",
        }
    }
}

struct DemoHelpOverlay {
    style: NodeGraphStyle,
    toggles: Arc<NodeGraphDemoOverlayToggles>,
    text_blobs: Vec<TextBlobId>,
}

impl DemoHelpOverlay {
    const PAD_PX: f32 = 10.0;
    const WIDTH_PX: f32 = 360.0;
    const HEIGHT_PX: f32 = 226.0;

    fn new(style: NodeGraphStyle, toggles: Arc<NodeGraphDemoOverlayToggles>) -> Self {
        Self {
            style,
            toggles,
            text_blobs: Vec::new(),
        }
    }

    fn rect(&self, bounds: Rect) -> Rect {
        let w = Self::WIDTH_PX.max(0.0);
        let h = Self::HEIGHT_PX.max(0.0);
        let x = bounds.origin.x.0 + Self::PAD_PX;
        let y = bounds.origin.y.0 + Self::PAD_PX;
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }
}

impl<H: UiHost> Widget<H> for DemoHelpOverlay {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let rect = self.rect(cx.bounds);
        let corner = self.style.context_menu_corner_radius.max(6.0);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(21_600),
            rect,
            background: fret_core::Paint::Solid(self.style.context_menu_background),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.context_menu_border),

            corner_radii: Corners::all(Px(corner)),
        });

        let text_style = self.style.controls_text_style.clone();
        let constraints = TextConstraints {
            max_width: Some(Px(rect.size.width.0 - 2.0 * Self::PAD_PX)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        let controls = if self.toggles.controls_in_panel() {
            "Panel"
        } else {
            "Floating"
        };
        let minimap = if self.toggles.minimap_in_panel() {
            "Panel"
        } else {
            "Floating"
        };
        let toolbars = if self.toggles.show_toolbars() {
            "On"
        } else {
            "Off"
        };
        let blackboard = if self.toggles.show_blackboard() {
            "On"
        } else {
            "Off"
        };

        let mut lines: Vec<String> = vec![
            "NodeGraph demo (built-ins):".to_string(),
            "• Tip: drag nodes by the header (title bar)".to_string(),
            "• Controls + MiniMap overlays (panel or floating)".to_string(),
            "• NodeToolbar + EdgeToolbar overlays (selection-driven)".to_string(),
            "• Background patterns: Cmd/Ctrl+B".to_string(),
            "• Toggle help: Cmd/Ctrl+H; toolbars: Cmd/Ctrl+T".to_string(),
            "• Toggle blackboard (symbols): Cmd/Ctrl+Shift+B".to_string(),
            "• Toggle placement: Cmd/Ctrl+Shift+C (controls), Cmd/Ctrl+Shift+M (minimap)"
                .to_string(),
            "• Log internals: Cmd/Ctrl+I; measured stores: Cmd/Ctrl+M".to_string(),
        ];
        lines.push(format!(
            "• Current: controls={controls}, minimap={minimap}, toolbars={toolbars}, blackboard={blackboard}"
        ));

        let mut cy = rect.origin.y.0 + Self::PAD_PX;
        for line in &lines {
            let (id, metrics) = cx
                .services
                .text()
                .prepare_str(line, &text_style, constraints);
            self.text_blobs.push(id);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(21_601),
                text: id,
                origin: Point::new(Px(rect.origin.x.0 + Self::PAD_PX), Px(cy)),
                paint: self.style.controls_text.into(),
                outline: None,
                shadow: None,
            });
            cy += metrics.size.height.0;
        }
    }
}

impl<H: UiHost> Widget<H> for DemoToolbarStrip {
    fn measure(&mut self, _cx: &mut MeasureCx<'_, H>) -> Size {
        let (w, h) = self.panel_size_px();
        Size::new(Px(w), Px(h))
    }

    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        self.compute_layout(bounds).panel.contains(position)
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.button_at(cx.bounds, *position);
                if hovered.is_some() {
                    cx.set_cursor_icon(CursorIcon::Pointer);
                }
                if hovered != self.hovered {
                    self.hovered = hovered;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let Some(btn) = self.button_at(cx.bounds, *position) else {
                    return;
                };
                self.pressed = Some(btn);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let pressed = self.pressed.take();
                cx.release_pointer_capture();
                if pressed.is_some() {
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                let Some(pressed) = pressed else {
                    return;
                };
                if self.button_at(cx.bounds, *position) == Some(pressed) {
                    self.dispatch_button(cx, pressed);
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let layout = self.compute_layout(cx.bounds);
        let corner = self.style.context_menu_corner_radius.max(6.0);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(21_500),
            rect: layout.panel,
            background: fret_core::Paint::Solid(self.style.context_menu_background),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.context_menu_border),

            corner_radii: Corners::all(Px(corner)),
        });

        let text_style = self.style.controls_text_style.clone();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        let (label_id, label_metrics) =
            cx.services
                .text()
                .prepare_str(self.label, &text_style, constraints);
        self.text_blobs.push(label_id);
        let lx = layout.panel.origin.x.0 + 8.0;
        let ly = layout.panel.origin.y.0
            + 0.5 * (layout.panel.size.height.0 - label_metrics.size.height.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(21_501),
            text: label_id,
            origin: Point::new(Px(lx), Px(ly)),
            paint: self.style.controls_text.into(),
            outline: None,
            shadow: None,
        });

        let buttons = [
            (DemoToolbarButton::Delete, layout.delete),
            (DemoToolbarButton::Fit, layout.fit),
        ];
        for (btn, rect) in buttons {
            let hovered = self.hovered == Some(btn);
            let pressed = self.pressed == Some(btn);
            let bg = if pressed {
                self.style.controls_active_background
            } else if hovered {
                self.style.controls_hover_background
            } else {
                Color::TRANSPARENT
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(21_501),
                rect,
                background: fret_core::Paint::Solid(bg),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px((corner - 2.0).max(4.0))),
            });

            let label = Self::label_for(btn);
            let (id, metrics) = cx
                .services
                .text()
                .prepare_str(label, &text_style, constraints);
            self.text_blobs.push(id);
            let tx = rect.origin.x.0 + 0.5 * (rect.size.width.0 - metrics.size.width.0);
            let ty = rect.origin.y.0 + 0.5 * (rect.size.height.0 - metrics.size.height.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(21_502),
                text: id,
                origin: Point::new(Px(tx), Px(ty)),
                paint: self.style.controls_text.into(),
                outline: None,
                shadow: None,
            });
        }
    }
}
