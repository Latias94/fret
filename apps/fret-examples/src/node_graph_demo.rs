use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_core::scene::DashPatternV1;
use fret_core::{Color, Px};
use fret_node::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use fret_node::io::{NodeGraphEditorConfig, NodeGraphViewState};
use fret_node::ui::{
    EdgePaintOverrideV1, NodeGraphDiagnosticsConfig, NodeGraphPaintOverridesMap,
    NodeGraphPaintOverridesRef, NodeGraphSurfaceBinding, node_graph_surface_in,
};
use serde_json::Value;

const ENV_DIAGNOSTICS: &str = "FRET_DIAG";
const ENV_WIRE_PAINT_COOKBOOK: &str = "FRET_NODE_GRAPH_DEMO_WIRE_PAINT_COOKBOOK";
const TEST_ID_CANVAS: &str = "node_graph.canvas";

#[derive(Clone)]
struct NodeGraphDemoView {
    surface: NodeGraphSurfaceBinding,
    paint_overrides: Option<NodeGraphPaintOverridesRef>,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("node-graph-demo")
        .window("node_graph_demo", (980.0, 720.0))
        .view::<NodeGraphDemoView>()?
        .run()?;
    Ok(())
}

impl View for NodeGraphDemoView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        let surface = NodeGraphSurfaceBinding::new(
            app.models_mut(),
            demo_graph(),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );
        let paint_overrides = demo_paint_overrides();
        Self {
            surface,
            paint_overrides,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        self.surface.observe_in(cx);

        let mut props = self.surface.surface_props();
        props.test_id = Some(Arc::<str>::from(TEST_ID_CANVAS));
        props.paint_overrides = self.paint_overrides.clone();
        if env_enabled(ENV_DIAGNOSTICS) {
            props.diagnostics = NodeGraphDiagnosticsConfig {
                key_actions_enabled: true,
                hover_tooltip_enabled: true,
            };
        }
        node_graph_surface_in(cx, props).into()
    }
}

fn env_enabled(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0" && v.trim() != "false")
}

fn demo_graph_ids() -> (GraphId, NodeId, NodeId, PortId, PortId, EdgeId) {
    let graph_id = GraphId::from_u128(0x6bfb_1b37_f0b9_4b62_8ac5_0f8c_0b8a_2f01);
    let src_node = NodeId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001);
    let dst_node = NodeId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0002);
    let src_out = PortId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0101);
    let dst_in = PortId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0102);
    let edge = EdgeId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0201);
    (graph_id, src_node, dst_node, src_out, dst_in, edge)
}

fn demo_graph() -> Graph {
    let (graph_id, src_node, dst_node, src_out, dst_in, edge) = demo_graph_ids();
    let mut graph = Graph::new(graph_id);

    graph.nodes.insert(
        src_node,
        Node {
            kind: NodeKindKey::new("demo.source"),
            kind_version: 1,
            pos: CanvasPoint { x: 80.0, y: 120.0 },
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
            ports: vec![src_out],
            data: Value::Null,
        },
    );
    graph.nodes.insert(
        dst_node,
        Node {
            kind: NodeKindKey::new("demo.sink"),
            kind_version: 1,
            pos: CanvasPoint { x: 520.0, y: 260.0 },
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
            ports: vec![dst_in],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        src_out,
        Port {
            node: src_node,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.ports.insert(
        dst_in,
        Port {
            node: dst_node,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: src_out,
            to: dst_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    graph
}

fn demo_paint_overrides() -> Option<NodeGraphPaintOverridesRef> {
    if !env_enabled(ENV_WIRE_PAINT_COOKBOOK) {
        return None;
    }

    let (_, _src_node, _dst_node, _src_out, _dst_in, edge) = demo_graph_ids();
    let overrides = Arc::new(NodeGraphPaintOverridesMap::default());
    overrides.set_edge_override(
        edge,
        Some(EdgePaintOverrideV1 {
            stroke_paint: Some(Color::from_srgb_hex_rgb(0x22c55e).into()),
            stroke_width_mul: Some(1.6),
            dash: Some(DashPatternV1::new(Px(6.0), Px(4.0), Px(0.0))),
        }),
    );

    Some(overrides)
}
