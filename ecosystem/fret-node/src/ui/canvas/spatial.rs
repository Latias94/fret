//! Derived spatial index helpers for large node graphs.
//!
//! This module indexes ports and edges into a coarse grid in canvas space.
//! It is intended as a UI-only acceleration structure for hit-testing and interaction previews.

use fret_canvas::spatial::DefaultIndexWithBackrefs;
use fret_canvas::wires as canvas_wires;
use fret_core::{Point, Rect};
use std::collections::HashMap;

use crate::core::{EdgeId, Graph, NodeId, PortId};

use super::geometry::CanvasGeometry;

mod spatial_adjacency;
mod spatial_derived;
mod spatial_index;

/// Coarse spatial index for hit-testing/culling (canvas space).
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialIndex {
    nodes: DefaultIndexWithBackrefs<NodeId>,
    ports: DefaultIndexWithBackrefs<PortId>,
    edges: DefaultIndexWithBackrefs<EdgeId>,
}

impl Default for CanvasSpatialIndex {
    fn default() -> Self {
        Self::empty(1.0)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CanvasPortEdgeAdjacency {
    edges_by_port: HashMap<PortId, Vec<EdgeId>>,
}

/// Spatial-derived outputs used by the canvas widget.
///
/// This intentionally bundles multiple “derived” helpers that share the same invalidation key:
/// - the coarse spatial index (rect → candidate IDs),
/// - adjacency lookups for incremental edge updates (port → edges),
/// - edge AABB padding policy used for conservative indexing and updates.
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialDerived {
    index: CanvasSpatialIndex,
    port_edges: CanvasPortEdgeAdjacency,
    edge_aabb_pad_canvas: f32,
}

impl Default for CanvasSpatialDerived {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
        PortDirection, PortId, PortKey, PortKind,
    };
    use crate::io::NodeGraphNodeOrigin;
    use crate::ui::canvas::geometry::CanvasGeometry;
    use crate::ui::presenter::NodeGraphPresenter;
    use crate::ui::style::NodeGraphStyle;
    use fret_core::{Point, Px};
    use serde_json::Value;
    use std::sync::Arc;

    #[derive(Default)]
    struct StubPresenter;

    impl NodeGraphPresenter for StubPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("port")
        }
    }

    fn make_graph_two_ports_one_node() -> (Graph, NodeId, PortId, PortId) {
        let mut graph = Graph::default();
        let node_id = NodeId::new();
        let in_port = PortId::new();
        let out_port = PortId::new();

        graph.nodes.insert(
            node_id,
            Node {
                kind: NodeKindKey::new("test.node"),
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 20.0 },
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
                ports: vec![in_port, out_port],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            in_port,
            Port {
                node: node_id,
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
        graph.ports.insert(
            out_port,
            Port {
                node: node_id,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        (graph, node_id, in_port, out_port)
    }

    #[test]
    fn edges_for_port_is_deterministic_and_self_loop_is_not_duplicated() {
        let (mut graph, node_id, in_port, out_port) = make_graph_two_ports_one_node();

        let e1 = EdgeId::new();
        let e2 = EdgeId::new();
        let e3 = EdgeId::new();
        graph.edges.insert(
            e1,
            Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: in_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: out_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            e3,
            Edge {
                kind: EdgeKind::Data,
                from: in_port,
                to: out_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let style = NodeGraphStyle::default();
        let mut presenter = StubPresenter::default();
        let geom = CanvasGeometry::build_with_presenter(
            &graph,
            &[node_id],
            &style,
            1.0,
            NodeGraphNodeOrigin::default(),
            &mut presenter,
            None,
        );

        let index = CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);

        let out_edges = index.edges_for_port(out_port).expect("out port edges");
        assert!(out_edges.contains(&e1));
        assert!(out_edges.contains(&e2));
        assert!(out_edges.contains(&e3));
        assert_eq!(
            out_edges.iter().copied().filter(|e| *e == e2).count(),
            1,
            "self-loop edge should appear once for its endpoint port"
        );

        // Determinism: edges are accumulated in `graph.edges` iteration order (BTreeMap by EdgeId).
        let mut sorted = out_edges.to_vec();
        sorted.sort();
        assert_eq!(out_edges, sorted.as_slice());
    }

    #[test]
    fn build_clamps_negative_pad_to_zero() {
        let (graph, node_id, _in_port, _out_port) = make_graph_two_ports_one_node();

        let style = NodeGraphStyle::default();
        let mut presenter = StubPresenter::default();
        let geom = CanvasGeometry::build_with_presenter(
            &graph,
            &[node_id],
            &style,
            1.0,
            NodeGraphNodeOrigin::default(),
            &mut presenter,
            None,
        );

        let index = CanvasSpatialDerived::build(&graph, &geom, 1.0, -123.0, 64.0);
        let from = Point::new(Px(0.0), Px(0.0));
        let to = Point::new(Px(10.0), Px(0.0));

        let a = index.edge_aabb(from, to, 1.0);
        let b = canvas_wires::wire_aabb(from, to, 1.0, 0.0);
        assert_eq!(a, b);
    }
}
