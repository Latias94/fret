//! Derived spatial index helpers for large node graphs.
//!
//! This module indexes ports and edges into a coarse grid in canvas space.
//! It is intended as a UI-only acceleration structure for hit-testing and interaction previews.

use std::collections::HashMap;

use fret_canvas::wires as canvas_wires;
use fret_core::{Point, Rect};

use crate::core::{EdgeId, Graph, NodeId, PortId};

use super::geometry::CanvasGeometry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
    x: i32,
    y: i32,
}

fn cell_key(cell: Cell) -> u64 {
    let x = cell.x as u32 as u64;
    let y = cell.y as u32 as u64;
    (x << 32) | y
}

fn cell_range_around(pos: Point, cell_size: f32, radius: f32) -> (i32, i32, i32, i32) {
    let s = cell_size.max(1.0e-6);
    let r = radius.max(0.0);
    let min_x = ((pos.x.0 - r) / s).floor() as i32;
    let max_x = ((pos.x.0 + r) / s).floor() as i32;
    let min_y = ((pos.y.0 - r) / s).floor() as i32;
    let max_y = ((pos.y.0 + r) / s).floor() as i32;
    (min_x, max_x, min_y, max_y)
}

fn cell_range_for_aabb(
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    cell_size: f32,
) -> (i32, i32, i32, i32) {
    let s = cell_size.max(1.0e-6);
    let min_x = (min_x / s).floor() as i32;
    let max_x = (max_x / s).floor() as i32;
    let min_y = (min_y / s).floor() as i32;
    let max_y = (max_y / s).floor() as i32;
    (min_x, max_x, min_y, max_y)
}

fn wire_aabb(from: Point, to: Point, zoom: f32, pad: f32) -> (f32, f32, f32, f32) {
    let (c1, c2) = canvas_wires::wire_ctrl_points(from, to, zoom);
    let mut min_x = from.x.0.min(to.x.0).min(c1.x.0).min(c2.x.0);
    let mut max_x = from.x.0.max(to.x.0).max(c1.x.0).max(c2.x.0);
    let mut min_y = from.y.0.min(to.y.0).min(c1.y.0).min(c2.y.0);
    let mut max_y = from.y.0.max(to.y.0).max(c1.y.0).max(c2.y.0);

    min_x -= pad;
    min_y -= pad;
    max_x += pad;
    max_y += pad;
    (min_x, min_y, max_x, max_y)
}

/// Coarse grid index for ports and edges (canvas space).
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialIndex {
    cell_size: f32,
    nodes: HashMap<u64, Vec<NodeId>>,
    ports: HashMap<u64, Vec<PortId>>,
    edges: HashMap<u64, Vec<EdgeId>>,
}

impl CanvasSpatialIndex {
    pub(crate) fn empty() -> Self {
        Self {
            cell_size: 1.0,
            nodes: HashMap::new(),
            ports: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub(crate) fn build(
        graph: &Graph,
        geom: &CanvasGeometry,
        zoom: f32,
        max_hit_pad_canvas: f32,
    ) -> Self {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let cell_size = (256.0 / zoom).max(16.0 / zoom).max(1.0);
        let mut out = Self {
            cell_size,
            nodes: HashMap::new(),
            ports: HashMap::new(),
            edges: HashMap::new(),
        };

        // Index nodes in draw order so deterministic tie-breaking can be layered on top.
        for node_id in geom.order.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node_id) else {
                continue;
            };
            let b = node_geom.rect;
            let min_x = b.origin.x.0.min(b.origin.x.0 + b.size.width.0);
            let min_y = b.origin.y.0.min(b.origin.y.0 + b.size.height.0);
            let max_x = b.origin.x.0.max(b.origin.x.0 + b.size.width.0);
            let max_y = b.origin.y.0.max(b.origin.y.0 + b.size.height.0);
            let (cx0, cx1, cy0, cy1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, cell_size);
            for y in cy0..=cy1 {
                for x in cx0..=cx1 {
                    out.nodes
                        .entry(cell_key(Cell { x, y }))
                        .or_default()
                        .push(node_id);
                }
            }
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
                let b = handle.bounds;
                let min_x = b.origin.x.0.min(b.origin.x.0 + b.size.width.0);
                let min_y = b.origin.y.0.min(b.origin.y.0 + b.size.height.0);
                let max_x = b.origin.x.0.max(b.origin.x.0 + b.size.width.0);
                let max_y = b.origin.y.0.max(b.origin.y.0 + b.size.height.0);
                let (cx0, cx1, cy0, cy1) =
                    cell_range_for_aabb(min_x, min_y, max_x, max_y, cell_size);
                for y in cy0..=cy1 {
                    for x in cx0..=cx1 {
                        out.ports
                            .entry(cell_key(Cell { x, y }))
                            .or_default()
                            .push(port_id);
                    }
                }
            }
        }

        let pad = max_hit_pad_canvas.max(0.0);
        for (&edge_id, edge) in &graph.edges {
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let (min_x, min_y, max_x, max_y) = wire_aabb(from, to, zoom, pad);
            let (cx0, cx1, cy0, cy1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, cell_size);
            for y in cy0..=cy1 {
                for x in cx0..=cx1 {
                    out.edges
                        .entry(cell_key(Cell { x, y }))
                        .or_default()
                        .push(edge_id);
                }
            }
        }

        out
    }

    pub(crate) fn query_ports(&self, pos: Point, radius: f32, out: &mut Vec<PortId>) {
        out.clear();
        let (x0, x1, y0, y1) = cell_range_around(pos, self.cell_size, radius);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(ids) = self.ports.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(ids);
                }
            }
        }
    }

    pub(crate) fn query_edges(&self, pos: Point, radius: f32, out: &mut Vec<EdgeId>) {
        out.clear();
        let (x0, x1, y0, y1) = cell_range_around(pos, self.cell_size, radius);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(ids) = self.edges.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(ids);
                }
            }
        }
    }

    pub(crate) fn query_edges_in_rect(&self, rect: Rect, out: &mut Vec<EdgeId>) {
        out.clear();
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        let (x0, x1, y0, y1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, self.cell_size);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(ids) = self.edges.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(ids);
                }
            }
        }
        out.sort_unstable();
        out.dedup();
    }

    pub(crate) fn query_nodes_in_rect(&self, rect: Rect, out: &mut Vec<NodeId>) {
        out.clear();
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        let (x0, x1, y0, y1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, self.cell_size);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(ids) = self.nodes.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(ids);
                }
            }
        }
        out.sort_unstable();
        out.dedup();
    }
}

impl Default for CanvasSpatialIndex {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use crate::core::{
        CanvasPoint, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };

    use super::super::geometry::{CanvasGeometry, NodeGeometry, PortHandleGeometry};
    use super::CanvasSpatialIndex;

    #[test]
    fn port_query_hits_bounds_even_when_center_is_elsewhere() {
        let mut graph = Graph::new(GraphId::new());

        let node = NodeId::new();
        let port = PortId::new();

        graph.nodes.insert(
            node,
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
                collapsed: false,
                ports: vec![port],
                data: serde_json::Value::Null,
            },
        );
        graph.ports.insert(
            port,
            Port {
                node,
                key: PortKey::new("p"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );

        // Create a handle geometry where the bounds are far away from the center.
        // This catches regressions where the spatial index only inserts ports by `center`.
        let bounds = Rect::new(
            Point::new(Px(1024.0), Px(2048.0)),
            Size::new(Px(12.0), Px(10.0)),
        );
        let center = Point::new(Px(0.0), Px(0.0));

        let mut geom = CanvasGeometry::default();
        geom.order = vec![node];
        geom.node_rank.insert(node, 0);
        geom.nodes.insert(
            node,
            NodeGeometry {
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            },
        );
        geom.ports.insert(
            port,
            PortHandleGeometry {
                node,
                dir: PortDirection::In,
                center,
                bounds,
            },
        );

        let index = CanvasSpatialIndex::build(&graph, &geom, 1.0, 0.0);

        let mut out: Vec<PortId> = Vec::new();
        let query_pos = Point::new(Px(1028.0), Px(2051.0));
        index.query_ports(query_pos, 1.0, &mut out);

        assert!(out.contains(&port));
    }

    #[test]
    fn edge_query_in_rect_returns_candidate_edge() {
        let mut graph = Graph::new(GraphId::new());

        let a = NodeId::new();
        let b = NodeId::new();
        let out_port = PortId::new();
        let in_port = PortId::new();
        let edge = EdgeId::new();

        graph.nodes.insert(
            a,
            Node {
                kind: NodeKindKey::new("test.a"),
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
                collapsed: false,
                ports: vec![out_port],
                data: serde_json::Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind: NodeKindKey::new("test.b"),
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
                collapsed: false,
                ports: vec![in_port],
                data: serde_json::Value::Null,
            },
        );

        graph.ports.insert(
            out_port,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        graph.ports.insert(
            in_port,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );

        graph.edges.insert(
            edge,
            crate::core::Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: in_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let mut geom = CanvasGeometry::default();
        geom.order = vec![a, b];
        geom.node_rank.insert(a, 0);
        geom.node_rank.insert(b, 1);
        geom.nodes.insert(
            a,
            NodeGeometry {
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            },
        );
        geom.nodes.insert(
            b,
            NodeGeometry {
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            },
        );
        geom.ports.insert(
            out_port,
            PortHandleGeometry {
                node: a,
                dir: PortDirection::Out,
                center: Point::new(Px(0.0), Px(0.0)),
                bounds: Rect::new(Point::new(Px(-2.0), Px(-2.0)), Size::new(Px(4.0), Px(4.0))),
            },
        );
        geom.ports.insert(
            in_port,
            PortHandleGeometry {
                node: b,
                dir: PortDirection::In,
                center: Point::new(Px(1000.0), Px(0.0)),
                bounds: Rect::new(Point::new(Px(998.0), Px(-2.0)), Size::new(Px(4.0), Px(4.0))),
            },
        );

        let index = CanvasSpatialIndex::build(&graph, &geom, 1.0, 0.0);

        let mut out: Vec<EdgeId> = Vec::new();
        let rect = Rect::new(
            Point::new(Px(480.0), Px(-10.0)),
            Size::new(Px(40.0), Px(20.0)),
        );
        index.query_edges_in_rect(rect, &mut out);

        assert!(out.contains(&edge));
    }

    #[test]
    fn node_query_in_rect_returns_candidate_node() {
        let mut graph = Graph::new(GraphId::new());

        let a = NodeId::new();
        let b = NodeId::new();

        graph.nodes.insert(
            a,
            Node {
                kind: NodeKindKey::new("test.a"),
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
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind: NodeKindKey::new("test.b"),
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
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );

        let mut geom = CanvasGeometry::default();
        geom.order = vec![a, b];
        geom.node_rank.insert(a, 0);
        geom.node_rank.insert(b, 1);
        geom.nodes.insert(
            a,
            NodeGeometry {
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(40.0))),
            },
        );
        geom.nodes.insert(
            b,
            NodeGeometry {
                rect: Rect::new(
                    Point::new(Px(1000.0), Px(0.0)),
                    Size::new(Px(80.0), Px(40.0)),
                ),
            },
        );

        let index = CanvasSpatialIndex::build(&graph, &geom, 1.0, 0.0);

        let mut out: Vec<NodeId> = Vec::new();
        index.query_nodes_in_rect(
            Rect::new(
                Point::new(Px(-10.0), Px(-10.0)),
                Size::new(Px(120.0), Px(80.0)),
            ),
            &mut out,
        );

        assert!(out.contains(&a));
        assert!(!out.contains(&b));
    }
}
