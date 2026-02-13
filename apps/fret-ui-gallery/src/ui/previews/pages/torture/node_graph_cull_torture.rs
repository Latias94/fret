use super::super::super::super::*;

pub(in crate::ui) fn preview_node_graph_cull_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{Px, SemanticsRole};
    use fret_node::io::NodeGraphViewState;
    use fret_node::ui::NodeGraphCanvas;
    use fret_node::{
        Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity,
        PortDirection, PortId, PortKey, PortKind, TypeDesc,
    };
    use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    fn uuid_from_tag(tag: u64, ix: u64) -> uuid::Uuid {
        uuid::Uuid::from_u128(((tag as u128) << 64) | (ix as u128))
    }

    fn build_stress_graph(graph_id: GraphId, target_nodes: usize) -> Graph {
        let mut graph = Graph::new(graph_id);

        let add_nodes = target_nodes.saturating_sub(1) / 2;
        let float_nodes = add_nodes.saturating_add(1);

        let cols: usize = 64;
        let x_step = 360.0f32;
        let y_step = 220.0f32;

        let float_x_offset = -260.0f32;
        let float_y_offset = 40.0f32;

        let node_tag = u64::from_le_bytes(*b"NODEGRAF");
        let port_tag = u64::from_le_bytes(*b"PORTGRAF");
        let edge_tag = u64::from_le_bytes(*b"EDGEGRAF");

        let node_id = |ix: u64| NodeId(uuid_from_tag(node_tag, ix));
        let port_id = |ix: u64| PortId(uuid_from_tag(port_tag, ix));
        let edge_id = |ix: u64| EdgeId(uuid_from_tag(edge_tag, ix));

        let mut next_node_ix: u64 = 1;
        let mut next_port_ix: u64 = 1;
        let mut next_edge_ix: u64 = 1;

        let mut float_out_ports: Vec<PortId> = Vec::with_capacity(float_nodes);
        for i in 0..float_nodes {
            let node_id = {
                let id = node_id(next_node_ix);
                next_node_ix = next_node_ix.saturating_add(1);
                id
            };
            let port_out = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };

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
                    pos: fret_node::CanvasPoint { x, y },
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
            let node_id = {
                let id = node_id(next_node_ix);
                next_node_ix = next_node_ix.saturating_add(1);
                id
            };
            let port_a = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };
            let port_b = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };
            let port_out = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };

            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * x_step;
            let y = row as f32 * y_step;

            graph.nodes.insert(
                node_id,
                Node {
                    kind: NodeKindKey::new("demo.add"),
                    kind_version: 1,
                    pos: fret_node::CanvasPoint { x, y },
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

            let port_a_source = prev_out.unwrap_or(float_out_ports[0]);
            let port_b_source =
                float_out_ports[(i + 1).min(float_out_ports.len().saturating_sub(1))];

            let edge_a = edge_id(next_edge_ix);
            next_edge_ix = next_edge_ix.saturating_add(1);
            graph.edges.insert(
                edge_a,
                Edge {
                    kind: EdgeKind::Data,
                    from: port_a_source,
                    to: port_a,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            );

            let edge_b = edge_id(next_edge_ix);
            next_edge_ix = next_edge_ix.saturating_add(1);
            graph.edges.insert(
                edge_b,
                Edge {
                    kind: EdgeKind::Data,
                    from: port_b_source,
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

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a large node-graph canvas with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    #[derive(Default)]
    struct HarnessState {
        graph: Option<Model<Graph>>,
        view: Option<Model<NodeGraphViewState>>,
    }

    let existing = cx.with_state(HarnessState::default, |st| {
        match (st.graph.clone(), st.view.clone()) {
            (Some(graph), Some(view)) => Some((graph, view)),
            _ => None,
        }
    });

    let (graph, view) = if let Some((graph, view)) = existing {
        (graph, view)
    } else {
        let graph_id = GraphId::from_u128(1);
        let graph = build_stress_graph(graph_id, 8_000);
        let graph = cx.app.models_mut().insert(graph);
        let view = cx.app.models_mut().insert(NodeGraphViewState::default());

        cx.with_state(HarnessState::default, |st| {
            st.graph = Some(graph.clone());
            st.view = Some(view.clone());
        });

        (graph, view)
    };

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let graph = graph.clone();
            let view = view.clone();

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(520.0));

            let props = RetainedSubtreeProps::new::<App>(move |ui| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;
                let canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
                ui.create_node_retained(canvas)
            })
            .with_layout(layout);

            let subtree = cx.retained_subtree(props);
            vec![cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-node-graph-cull-root")),
                    ..Default::default()
                },
                |_cx| vec![subtree],
            )]
        });

    vec![header, surface]
}
