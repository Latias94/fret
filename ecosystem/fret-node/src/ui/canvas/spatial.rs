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

/// Coarse grid index for ports and edges (canvas space).
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialIndex {
    nodes: DefaultIndexWithBackrefs<NodeId>,
    ports: DefaultIndexWithBackrefs<PortId>,
    edges: DefaultIndexWithBackrefs<EdgeId>,
    edge_aabb_pad_canvas: f32,
    edges_by_port: HashMap<PortId, Vec<EdgeId>>,
}

impl CanvasSpatialIndex {
    pub(crate) fn empty() -> Self {
        Self {
            nodes: DefaultIndexWithBackrefs::new(1.0),
            ports: DefaultIndexWithBackrefs::new(1.0),
            edges: DefaultIndexWithBackrefs::new(1.0),
            edge_aabb_pad_canvas: 0.0,
            edges_by_port: HashMap::new(),
        }
    }

    pub(crate) fn build(
        graph: &Graph,
        geom: &CanvasGeometry,
        zoom: f32,
        max_hit_pad_canvas: f32,
        cell_size_canvas: f32,
    ) -> Self {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let cell_size = if cell_size_canvas.is_finite() && cell_size_canvas > 0.0 {
            cell_size_canvas
        } else {
            (256.0 / zoom).max(16.0 / zoom).max(1.0)
        };

        let mut nodes = DefaultIndexWithBackrefs::new(cell_size);
        let mut ports = DefaultIndexWithBackrefs::new(cell_size);
        let mut edges = DefaultIndexWithBackrefs::new(cell_size);
        let mut edges_by_port: HashMap<PortId, Vec<EdgeId>> = HashMap::new();

        // Index nodes in draw order so deterministic tie-breaking can be layered on top.
        for node_id in geom.order.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node_id) else {
                continue;
            };
            nodes.insert_rect(node_id, node_geom.rect);
        }

        // Insert ports in node draw order so that tie-breaking (when distances match) can prefer
        // the top-most node without relying on map iteration order.
        for node_id in geom.order.iter().copied() {
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };
            for port_id in node.ports.iter().copied() {
                let Some(handle) = geom.ports.get(&port_id) else {
                    continue;
                };
                ports.insert_rect(port_id, handle.bounds);
            }
        }

        let pad = max_hit_pad_canvas.max(0.0);
        for (&edge_id, edge) in &graph.edges {
            if edge.from == edge.to {
                edges_by_port.entry(edge.from).or_default().push(edge_id);
            } else {
                edges_by_port.entry(edge.from).or_default().push(edge_id);
                edges_by_port.entry(edge.to).or_default().push(edge_id);
            }

            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let rect = canvas_wires::wire_aabb(from, to, zoom, pad);
            edges.insert_rect(edge_id, rect);
        }

        Self {
            nodes,
            ports,
            edges,
            edge_aabb_pad_canvas: pad,
            edges_by_port,
        }
    }

    pub(crate) fn query_ports_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<PortId>,
    ) -> &'a [PortId] {
        self.ports.query_radius_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<EdgeId>,
    ) -> &'a [EdgeId] {
        self.edges.query_radius_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_in_rect(&self, rect: Rect, out: &mut Vec<EdgeId>) {
        let _ = self.edges.query_rect_sorted_dedup(rect, out);
    }

    pub(crate) fn query_nodes_in_rect(&self, rect: Rect, out: &mut Vec<NodeId>) {
        let _ = self.nodes.query_rect_sorted_dedup(rect, out);
    }

    pub(crate) fn edge_aabb(&self, from: Point, to: Point, zoom: f32) -> Rect {
        canvas_wires::wire_aabb(from, to, zoom, self.edge_aabb_pad_canvas)
    }

    pub(crate) fn edges_for_port(&self, port: PortId) -> Option<&[EdgeId]> {
        self.edges_by_port.get(&port).map(|v| v.as_slice())
    }

    pub(crate) fn update_node_rect(&mut self, node: NodeId, rect: Rect) {
        self.nodes.update_rect(node, rect);
    }

    pub(crate) fn update_port_rect(&mut self, port: PortId, rect: Rect) {
        self.ports.update_rect(port, rect);
    }

    pub(crate) fn update_edge_rect(&mut self, edge: EdgeId, rect: Rect) {
        self.edges.update_rect(edge, rect);
    }
}

impl Default for CanvasSpatialIndex {
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
        );

        let index = CanvasSpatialIndex::build(&graph, &geom, 1.0, 0.0, 64.0);

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
        );

        let index = CanvasSpatialIndex::build(&graph, &geom, 1.0, -123.0, 64.0);
        let from = Point::new(Px(0.0), Px(0.0));
        let to = Point::new(Px(10.0), Px(0.0));

        let a = index.edge_aabb(from, to, 1.0);
        let b = canvas_wires::wire_aabb(from, to, 1.0, 0.0);
        assert_eq!(a, b);
    }
}
