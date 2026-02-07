use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use serde_json::Value;

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::geometry::{CanvasGeometry, NodeGeometry, PortHandleGeometry};
use crate::ui::canvas::spatial::CanvasSpatialDerived;
use crate::ui::presenter::NodeGraphPresenter;
use crate::ui::style::NodeGraphStyle;

use super::prelude::NodeGraphCanvas;

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

fn assert_near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-5, "{a} != {b}");
}

fn make_graph_one_node_two_ports() -> (Graph, NodeId, PortId, PortId) {
    let mut graph = Graph::default();
    let node_id = NodeId::new();
    let in_port = PortId::new();
    let out_port = PortId::new();

    graph.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
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
fn update_ports_for_node_rect_change_keeps_ports_pinned_to_sides() {
    let (graph, node_id, in_port, out_port) = make_graph_one_node_two_ports();

    let prev_rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(100.0), Px(50.0)),
    );
    let next_rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(200.0), Px(50.0)),
    );

    let mut geom = CanvasGeometry::default();
    geom.order = vec![node_id];
    geom.node_rank.insert(node_id, 0);
    geom.nodes.insert(node_id, NodeGeometry { rect: prev_rect });

    let pin_bounds = Size::new(Px(10.0), Px(10.0));
    let in_center_prev = Point::new(Px(prev_rect.origin.x.0), Px(prev_rect.origin.y.0 + 30.0));
    let out_center_prev = Point::new(
        Px(prev_rect.origin.x.0 + prev_rect.size.width.0),
        Px(prev_rect.origin.y.0 + 30.0),
    );
    geom.ports.insert(
        in_port,
        PortHandleGeometry {
            node: node_id,
            dir: PortDirection::In,
            center: in_center_prev,
            bounds: Rect::new(
                Point::new(Px(in_center_prev.x.0 - 5.0), Px(in_center_prev.y.0 - 5.0)),
                pin_bounds,
            ),
        },
    );
    geom.ports.insert(
        out_port,
        PortHandleGeometry {
            node: node_id,
            dir: PortDirection::Out,
            center: out_center_prev,
            bounds: Rect::new(
                Point::new(Px(out_center_prev.x.0 - 5.0), Px(out_center_prev.y.0 - 5.0)),
                pin_bounds,
            ),
        },
    );

    let mut index = CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);

    NodeGraphCanvas::update_ports_for_node_rect_change(
        &mut geom,
        &mut index,
        node_id,
        prev_rect,
        next_rect,
        &[in_port, out_port],
    );

    let in_after = geom.ports.get(&in_port).unwrap();
    assert_near(in_after.center.x.0, next_rect.origin.x.0);
    assert_near(in_after.center.y.0, in_center_prev.y.0);

    let out_after = geom.ports.get(&out_port).unwrap();
    assert_near(
        out_after.center.x.0,
        next_rect.origin.x.0 + next_rect.size.width.0,
    );
    assert_near(out_after.center.y.0, out_center_prev.y.0);

    let out_bounds_center_x = out_after.bounds.origin.x.0 + 0.5 * out_after.bounds.size.width.0;
    let out_bounds_center_y = out_after.bounds.origin.y.0 + 0.5 * out_after.bounds.size.height.0;
    assert_near(out_bounds_center_x, out_after.center.x.0);
    assert_near(out_bounds_center_y, out_after.center.y.0);
}

#[test]
fn update_edges_for_ports_dedups_edge_ids_and_updates_index() {
    let (mut graph, node_id, in_port, out_port) = make_graph_one_node_two_ports();

    let e1 = EdgeId::new();
    let e2 = EdgeId::new();
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

    let style = NodeGraphStyle::default();
    let mut presenter = StubPresenter::default();
    let mut geom = CanvasGeometry::build_with_presenter(
        &graph,
        &[node_id],
        &style,
        1.0,
        NodeGraphNodeOrigin::default(),
        &mut presenter,
    );
    let mut index = CanvasSpatialDerived::build(&graph, &geom, 1.0, 2.0, 64.0);

    // Force a geometry change so updating edges must relocate their AABBs.
    let out_handle = geom.ports.get_mut(&out_port).unwrap();
    out_handle.center.x = Px(out_handle.center.x.0 + 200.0);
    out_handle.bounds.origin.x = Px(out_handle.bounds.origin.x.0 + 200.0);

    let old_query_rect = Rect::new(
        Point::new(Px(-1000.0), Px(-1000.0)),
        Size::new(Px(2000.0), Px(2000.0)),
    );
    let mut old_edges = Vec::new();
    index.query_edges_in_rect(old_query_rect, &mut old_edges);
    assert!(old_edges.contains(&e1));
    assert!(old_edges.contains(&e2));

    NodeGraphCanvas::update_edges_for_ports(
        &mut geom,
        &mut index,
        1.0,
        &[out_port, in_port],
        |edge_ids| {
            let mut sorted = edge_ids.to_vec();
            sorted.sort();
            sorted.dedup();
            assert_eq!(
                edge_ids,
                sorted.as_slice(),
                "expected sorted+deduped edge ids"
            );

            edge_ids
                .iter()
                .filter_map(|edge_id| graph.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to)))
                .collect::<Vec<_>>()
        },
    );

    // The old "giant rect" should still contain them, but now a small rect around the *new*
    // geometry should include at least one edge and the index should be internally consistent.
    let from = geom.port_center(out_port).unwrap();
    let to = geom.port_center(in_port).unwrap();
    let aabb = index.edge_aabb(from, to, 1.0);
    let mut hits = Vec::new();
    index.query_edges_in_rect(aabb, &mut hits);
    assert!(hits.contains(&e1));
}
