use std::sync::Arc;

use fret::prelude::*;
use fret_core::scene::DashPatternV1;
use fret_core::{Color, Px};
use fret_node::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use fret_node::io::NodeGraphViewState;
use fret_node::ui::{
    EdgePaintOverrideV1, NodeGraphPaintOverridesMap, NodeGraphPaintOverridesRef,
    NodeGraphSurfacePaintOnlyProps, node_graph_surface_paint_only,
};
use serde_json::Value;

const ENV_WIRE_PAINT_COOKBOOK: &str = "FRET_NODE_GRAPH_DEMO_WIRE_PAINT_COOKBOOK";
const TEST_ID_CANVAS: &str = "node_graph.canvas";

#[derive(Clone)]
struct NodeGraphDemoState {
    graph: Model<Graph>,
    view: Model<NodeGraphViewState>,
    paint_overrides: Option<NodeGraphPaintOverridesRef>,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("node-graph-demo")
        .window("node_graph_demo", (980.0, 720.0))
        .ui(init_window, view)?
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> NodeGraphDemoState {
    let graph = app.models_mut().insert(demo_graph());
    let view = app.models_mut().insert(NodeGraphViewState::default());
    let paint_overrides = demo_paint_overrides();
    NodeGraphDemoState {
        graph,
        view,
        paint_overrides,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut NodeGraphDemoState) -> fret::ViewElements {
    cx.observe_model(&st.graph, Invalidation::Paint);
    cx.observe_model(&st.view, Invalidation::Paint);

    let mut props = NodeGraphSurfacePaintOnlyProps::new(st.graph.clone(), st.view.clone());
    props.test_id = Some(Arc::<str>::from(TEST_ID_CANVAS));
    props.paint_overrides = st.paint_overrides.clone();
    node_graph_surface_paint_only(cx, props).into()
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
