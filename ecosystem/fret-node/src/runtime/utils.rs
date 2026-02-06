//! Headless helper utilities (XyFlow-style graph helpers).
//!
//! XyFlow exposes a set of convenience utilities in `@xyflow/system/src/utils/*` for common
//! graph queries (e.g. "incomers/outgoers", "connected edges", "nodes inside rect", bounds).
//!
//! In fret-node, the canonical document (`core::Graph`) is port-based (edges connect ports), so
//! these helpers are built on top of `runtime::lookups::NodeGraphLookups` which provides a stable,
//! headless-safe adjacency surface.

use crate::core::{CanvasPoint, CanvasRect, CanvasSize, EdgeId, NodeId};
use crate::runtime::lookups::{ConnectionSide, NodeGraphLookups};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeInclusion {
    /// Include nodes that intersect the query rect.
    Partial,
    /// Include nodes only when fully contained within the query rect.
    Full,
}

#[derive(Debug, Clone, Copy)]
pub struct GetNodesBoundsOptions {
    /// Node origin (anchor) used to interpret `Node.pos`.
    ///
    /// - `(0.0, 0.0)` means `pos` is top-left.
    /// - `(0.5, 0.5)` means `pos` is center.
    pub node_origin: (f32, f32),
    /// Whether to include hidden nodes.
    pub include_hidden: bool,
    /// Fallback size to use when a node has no explicit size.
    ///
    /// When `None`, nodes without a size are skipped.
    pub fallback_size: Option<CanvasSize>,
}

impl Default for GetNodesBoundsOptions {
    fn default() -> Self {
        Self {
            node_origin: (0.0, 0.0),
            include_hidden: false,
            fallback_size: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GetNodesInsideOptions {
    pub inclusion: NodeInclusion,
    pub node_origin: (f32, f32),
    pub include_hidden: bool,
    pub fallback_size: Option<CanvasSize>,
}

impl Default for GetNodesInsideOptions {
    fn default() -> Self {
        Self {
            inclusion: NodeInclusion::Partial,
            node_origin: (0.0, 0.0),
            include_hidden: false,
            fallback_size: None,
        }
    }
}

/// Returns the nodes connected as *targets* of the given node's outgoing edges.
pub fn get_outgoers(lookups: &NodeGraphLookups, node: NodeId) -> Vec<NodeId> {
    let Some(conns) = lookups.connections_for_node_side(node, ConnectionSide::Source) else {
        return Vec::new();
    };
    let mut out: Vec<NodeId> = conns.values().map(|c| c.target_node).collect();
    out.sort();
    out.dedup();
    out
}

/// Returns the top-left position for a node, taking node origin into account.
///
/// This mirrors XyFlow's `getNodePositionWithOrigin` utility.
pub fn get_node_position_with_origin(
    lookups: &NodeGraphLookups,
    node: NodeId,
    node_origin: (f32, f32),
    fallback_size: Option<CanvasSize>,
) -> Option<CanvasPoint> {
    let entry = lookups.node_lookup.get(&node)?;
    let (ox, oy) = normalize_node_origin(node_origin);
    let (x0, y0, _x1, _y1) = node_rect_xy(entry.pos, entry.size, (ox, oy), fallback_size)?;
    Some(CanvasPoint { x: x0, y: y0 })
}

/// Returns the node's canvas-space bounding rect.
pub fn get_node_rect(
    lookups: &NodeGraphLookups,
    node: NodeId,
    node_origin: (f32, f32),
    fallback_size: Option<CanvasSize>,
) -> Option<CanvasRect> {
    let entry = lookups.node_lookup.get(&node)?;
    let (ox, oy) = normalize_node_origin(node_origin);
    let (x0, y0, x1, y1) = node_rect_xy(entry.pos, entry.size, (ox, oy), fallback_size)?;
    Some(CanvasRect {
        origin: CanvasPoint { x: x0, y: y0 },
        size: CanvasSize {
            width: (x1 - x0).max(0.0),
            height: (y1 - y0).max(0.0),
        },
    })
}

/// Returns the nodes connected as *sources* of the given node's incoming edges.
pub fn get_incomers(lookups: &NodeGraphLookups, node: NodeId) -> Vec<NodeId> {
    let Some(conns) = lookups.connections_for_node_side(node, ConnectionSide::Target) else {
        return Vec::new();
    };
    let mut out: Vec<NodeId> = conns.values().map(|c| c.source_node).collect();
    out.sort();
    out.dedup();
    out
}

/// Returns all edges incident to the given node (both directions).
pub fn get_connected_edges(lookups: &NodeGraphLookups, node: NodeId) -> Vec<EdgeId> {
    let Some(conns) = lookups.connections_for_node(node) else {
        return Vec::new();
    };
    let mut out: Vec<EdgeId> = conns.values().map(|c| c.edge).collect();
    out.sort();
    out.dedup();
    out
}

/// Returns all edges connected to any node in the given set.
///
/// This matches the intent of XyFlow's `getConnectedEdges(nodes, edges)` helper, but uses
/// `NodeGraphLookups` rather than scanning an edge array.
pub fn get_connected_edges_for_nodes(
    lookups: &NodeGraphLookups,
    nodes: impl IntoIterator<Item = NodeId>,
) -> Vec<EdgeId> {
    let mut out: Vec<EdgeId> = Vec::new();
    for node in nodes {
        out.extend(get_connected_edges(lookups, node));
    }
    out.sort();
    out.dedup();
    out
}

/// Computes the bounding rect enclosing the given nodes.
///
/// Returns `None` when no nodes contribute a valid rect (e.g. all nodes are missing sizes and
/// no `fallback_size` is provided).
pub fn get_nodes_bounds(
    lookups: &NodeGraphLookups,
    nodes: impl IntoIterator<Item = NodeId>,
    options: GetNodesBoundsOptions,
) -> Option<CanvasRect> {
    let (ox, oy) = normalize_node_origin(options.node_origin);
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for node in nodes {
        let Some(entry) = lookups.node_lookup.get(&node) else {
            continue;
        };
        if !options.include_hidden && entry.hidden {
            continue;
        }
        let Some((x0, y0, x1, y1)) =
            node_rect_xy(entry.pos, entry.size, (ox, oy), options.fallback_size)
        else {
            continue;
        };
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some(CanvasRect {
        origin: CanvasPoint { x: min_x, y: min_y },
        size: CanvasSize {
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        },
    })
}

/// Returns the nodes that are inside the given query rect.
pub fn get_nodes_inside(
    lookups: &NodeGraphLookups,
    rect: CanvasRect,
    options: GetNodesInsideOptions,
) -> Vec<NodeId> {
    let (ox, oy) = normalize_node_origin(options.node_origin);
    let (rx0, ry0, rx1, ry1) = rect_xy(rect);
    if !rx0.is_finite() || !ry0.is_finite() || !rx1.is_finite() || !ry1.is_finite() {
        return Vec::new();
    }

    let mut out: Vec<NodeId> = Vec::new();
    for (node, entry) in &lookups.node_lookup {
        if !options.include_hidden && entry.hidden {
            continue;
        }
        let Some((x0, y0, x1, y1)) =
            node_rect_xy(entry.pos, entry.size, (ox, oy), options.fallback_size)
        else {
            continue;
        };

        let keep = match options.inclusion {
            NodeInclusion::Partial => rects_intersect((rx0, ry0, rx1, ry1), (x0, y0, x1, y1)),
            NodeInclusion::Full => rect_contains((rx0, ry0, rx1, ry1), (x0, y0, x1, y1)),
        };
        if keep {
            out.push(*node);
        }
    }

    out.sort();
    out
}

fn normalize_node_origin(origin: (f32, f32)) -> (f32, f32) {
    let mut ox = origin.0;
    let mut oy = origin.1;
    if !ox.is_finite() {
        ox = 0.0;
    }
    if !oy.is_finite() {
        oy = 0.0;
    }
    (ox.clamp(0.0, 1.0), oy.clamp(0.0, 1.0))
}

fn rect_xy(rect: CanvasRect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let w = rect.size.width.max(0.0);
    let h = rect.size.height.max(0.0);
    (x0, y0, x0 + w, y0 + h)
}

fn node_rect_xy(
    pos: CanvasPoint,
    size: Option<CanvasSize>,
    node_origin: (f32, f32),
    fallback_size: Option<CanvasSize>,
) -> Option<(f32, f32, f32, f32)> {
    let size = size.or(fallback_size)?;
    let w = size.width;
    let h = size.height;
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return None;
    }
    if !pos.x.is_finite() || !pos.y.is_finite() {
        return None;
    }
    let (ox, oy) = node_origin;
    let x0 = pos.x - ox * w;
    let y0 = pos.y - oy * h;
    Some((x0, y0, x0 + w, y0 + h))
}

fn rects_intersect(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    let (ax0, ay0, ax1, ay1) = a;
    let (bx0, by0, bx1, by1) = b;
    ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
}

fn rect_contains(outer: (f32, f32, f32, f32), inner: (f32, f32, f32, f32)) -> bool {
    let (ox0, oy0, ox1, oy1) = outer;
    let (ix0, iy0, ix1, iy1) = inner;
    ix0 >= ox0 && iy0 >= oy0 && ix1 <= ox1 && iy1 <= oy1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Edge, EdgeKind, Graph, GraphId, Node, NodeKindKey, Port, PortCapacity, PortDirection,
        PortId, PortKey, PortKind,
    };

    fn node_at(pos: CanvasPoint, size: Option<CanvasSize>) -> Node {
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        }
    }

    fn out_port(node: NodeId) -> (PortId, Port) {
        let pid = PortId::new();
        (
            pid,
            Port {
                node,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        )
    }

    fn in_port(node: NodeId, key: &str) -> (PortId, Port) {
        let pid = PortId::new();
        (
            pid,
            Port {
                node,
                key: PortKey::new(key),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        )
    }

    #[test]
    fn outgoers_incomers_connected_edges_are_derived_from_connections() {
        let mut g = Graph::new(GraphId::from_u128(1));

        let a = NodeId::new();
        let b = NodeId::new();
        let c = NodeId::new();

        g.nodes
            .insert(a, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None));
        g.nodes
            .insert(b, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None));
        g.nodes
            .insert(c, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None));

        let (a_out_id, a_out) = out_port(a);
        let (b_in_id, b_in) = in_port(b, "in0");
        let (c_in_id, c_in) = in_port(c, "in0");
        g.ports.insert(a_out_id, a_out);
        g.ports.insert(b_in_id, b_in);
        g.ports.insert(c_in_id, c_in);

        let e1 = EdgeId::new();
        let e2 = EdgeId::new();
        g.edges.insert(
            e1,
            Edge {
                kind: EdgeKind::Data,
                from: a_out_id,
                to: b_in_id,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        g.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: a_out_id,
                to: c_in_id,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&g);

        let mut expected_outgoers = vec![b, c];
        expected_outgoers.sort();
        assert_eq!(get_outgoers(&lookups, a), expected_outgoers);
        assert_eq!(get_incomers(&lookups, b), vec![a]);
        assert_eq!(get_incomers(&lookups, c), vec![a]);

        let connected = get_connected_edges(&lookups, a);
        assert_eq!(connected.len(), 2);
        assert!(connected.contains(&e1));
        assert!(connected.contains(&e2));

        let connected_for_nodes = get_connected_edges_for_nodes(&lookups, [b, c]);
        let mut expected = vec![e1, e2];
        expected.sort();
        assert_eq!(connected_for_nodes, expected);
    }

    #[test]
    fn helpers_are_deterministic_under_insertion_order_variance() {
        fn build_graph(insert_a_first: bool) -> (Graph, NodeId, NodeId, NodeId, EdgeId, EdgeId) {
            let mut g = Graph::new(GraphId::from_u128(1));

            let a = NodeId(uuid::Uuid::from_u128(1));
            let b = NodeId(uuid::Uuid::from_u128(2));
            let c = NodeId(uuid::Uuid::from_u128(3));

            let nodes = [
                (a, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None)),
                (b, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None)),
                (c, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None)),
            ];
            if insert_a_first {
                for (id, node) in nodes {
                    g.nodes.insert(id, node);
                }
            } else {
                for (id, node) in nodes.into_iter().rev() {
                    g.nodes.insert(id, node);
                }
            }

            let a_out_id = PortId(uuid::Uuid::from_u128(10));
            let b_in_id = PortId(uuid::Uuid::from_u128(11));
            let c_in_id = PortId(uuid::Uuid::from_u128(12));
            let a_out = Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            };
            let b_in = Port {
                node: b,
                key: PortKey::new("in0"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            };
            let c_in = Port {
                node: c,
                key: PortKey::new("in0"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            };

            if insert_a_first {
                g.ports.insert(a_out_id, a_out);
                g.ports.insert(b_in_id, b_in);
                g.ports.insert(c_in_id, c_in);
            } else {
                g.ports.insert(c_in_id, c_in);
                g.ports.insert(b_in_id, b_in);
                g.ports.insert(a_out_id, a_out);
            }

            let e1 = EdgeId(uuid::Uuid::from_u128(20));
            let e2 = EdgeId(uuid::Uuid::from_u128(21));
            if insert_a_first {
                g.edges.insert(
                    e1,
                    Edge {
                        kind: EdgeKind::Data,
                        from: a_out_id,
                        to: b_in_id,
                        selectable: None,
                        deletable: None,
                        reconnectable: None,
                    },
                );
                g.edges.insert(
                    e2,
                    Edge {
                        kind: EdgeKind::Data,
                        from: a_out_id,
                        to: c_in_id,
                        selectable: None,
                        deletable: None,
                        reconnectable: None,
                    },
                );
            } else {
                g.edges.insert(
                    e2,
                    Edge {
                        kind: EdgeKind::Data,
                        from: a_out_id,
                        to: c_in_id,
                        selectable: None,
                        deletable: None,
                        reconnectable: None,
                    },
                );
                g.edges.insert(
                    e1,
                    Edge {
                        kind: EdgeKind::Data,
                        from: a_out_id,
                        to: b_in_id,
                        selectable: None,
                        deletable: None,
                        reconnectable: None,
                    },
                );
            }

            (g, a, b, c, e1, e2)
        }

        let (g1, a1, b1, c1, e11, e21) = build_graph(true);
        let (g2, a2, b2, c2, e12, e22) = build_graph(false);
        assert_eq!((a1, b1, c1, e11, e21), (a2, b2, c2, e12, e22));

        let mut l1 = NodeGraphLookups::default();
        l1.rebuild_from(&g1);
        let mut l2 = NodeGraphLookups::default();
        l2.rebuild_from(&g2);

        assert_eq!(get_outgoers(&l1, a1), get_outgoers(&l2, a2));
        assert_eq!(get_incomers(&l1, b1), get_incomers(&l2, b2));
        assert_eq!(get_incomers(&l1, c1), get_incomers(&l2, c2));

        let mut expected_edges = vec![e11, e21];
        expected_edges.sort();
        let mut actual_edges_1 = get_connected_edges(&l1, a1);
        actual_edges_1.sort();
        let mut actual_edges_2 = get_connected_edges(&l2, a2);
        actual_edges_2.sort();
        assert_eq!(actual_edges_1, expected_edges);
        assert_eq!(actual_edges_2, expected_edges);
    }

    #[test]
    fn outgoers_and_incomers_include_self_for_self_loops_and_dedup() {
        let mut g = Graph::new(GraphId::from_u128(1));

        let a = NodeId::new();
        let (a_out_id, a_out) = out_port(a);
        let (a_in_id, a_in) = in_port(a, "in0");

        g.nodes
            .insert(a, node_at(CanvasPoint { x: 0.0, y: 0.0 }, None));
        g.ports.insert(a_out_id, a_out);
        g.ports.insert(a_in_id, a_in);

        // Two self-loop edges should still dedup to a single node in outgoers/incomers.
        let e1 = EdgeId::new();
        let e2 = EdgeId::new();
        g.edges.insert(
            e1,
            Edge {
                kind: EdgeKind::Data,
                from: a_out_id,
                to: a_in_id,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        g.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: a_out_id,
                to: a_in_id,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&g);

        assert_eq!(get_outgoers(&lookups, a), vec![a]);
        assert_eq!(get_incomers(&lookups, a), vec![a]);

        let connected = get_connected_edges(&lookups, a);
        assert_eq!(connected.len(), 2);
        assert!(connected.contains(&e1));
        assert!(connected.contains(&e2));
    }

    #[test]
    fn get_node_position_with_origin_matches_bounds_top_left() {
        let mut g = Graph::new(GraphId::from_u128(1));
        let a = NodeId::new();
        g.nodes.insert(
            a,
            node_at(
                CanvasPoint { x: 20.0, y: 10.0 },
                Some(CanvasSize {
                    width: 10.0,
                    height: 6.0,
                }),
            ),
        );

        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&g);

        let p = get_node_position_with_origin(&lookups, a, (0.5, 0.5), None).expect("pos");
        assert!((p.x - 15.0).abs() <= 1.0e-6);
        assert!((p.y - 7.0).abs() <= 1.0e-6);
    }

    #[test]
    fn get_node_rect_is_consistent_with_get_nodes_bounds_singleton() {
        let mut g = Graph::new(GraphId::from_u128(1));
        let a = NodeId::new();
        g.nodes.insert(
            a,
            node_at(
                CanvasPoint { x: 20.0, y: 10.0 },
                Some(CanvasSize {
                    width: 10.0,
                    height: 6.0,
                }),
            ),
        );

        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&g);

        let rect = get_node_rect(&lookups, a, (0.5, 0.5), None).expect("rect");
        let bounds = get_nodes_bounds(
            &lookups,
            [a],
            GetNodesBoundsOptions {
                node_origin: (0.5, 0.5),
                include_hidden: true,
                fallback_size: None,
            },
        )
        .expect("bounds");

        assert!((rect.origin.x - bounds.origin.x).abs() <= 1.0e-6);
        assert!((rect.origin.y - bounds.origin.y).abs() <= 1.0e-6);
        assert!((rect.size.width - bounds.size.width).abs() <= 1.0e-6);
        assert!((rect.size.height - bounds.size.height).abs() <= 1.0e-6);
    }

    #[test]
    fn get_nodes_bounds_respects_node_origin() {
        let mut g = Graph::new(GraphId::from_u128(1));
        let a = NodeId::new();
        let b = NodeId::new();

        g.nodes.insert(
            a,
            node_at(
                CanvasPoint { x: 0.0, y: 0.0 },
                Some(CanvasSize {
                    width: 10.0,
                    height: 10.0,
                }),
            ),
        );
        g.nodes.insert(
            b,
            node_at(
                CanvasPoint { x: 20.0, y: 5.0 },
                Some(CanvasSize {
                    width: 5.0,
                    height: 5.0,
                }),
            ),
        );

        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&g);

        let bounds_top_left = get_nodes_bounds(
            &lookups,
            [a, b],
            GetNodesBoundsOptions {
                node_origin: (0.0, 0.0),
                include_hidden: true,
                fallback_size: None,
            },
        )
        .expect("bounds");
        assert!((bounds_top_left.origin.x - 0.0).abs() <= 1.0e-6);
        assert!((bounds_top_left.origin.y - 0.0).abs() <= 1.0e-6);
        assert!((bounds_top_left.size.width - 25.0).abs() <= 1.0e-6);
        assert!((bounds_top_left.size.height - 10.0).abs() <= 1.0e-6);

        let bounds_center = get_nodes_bounds(
            &lookups,
            [a, b],
            GetNodesBoundsOptions {
                node_origin: (0.5, 0.5),
                include_hidden: true,
                fallback_size: None,
            },
        )
        .expect("bounds");
        assert!((bounds_center.origin.x - (-5.0)).abs() <= 1.0e-6);
        assert!((bounds_center.origin.y - (-5.0)).abs() <= 1.0e-6);
        assert!((bounds_center.size.width - 27.5).abs() <= 1.0e-6);
        assert!((bounds_center.size.height - 12.5).abs() <= 1.0e-6);
    }

    #[test]
    fn get_nodes_inside_supports_partial_vs_full_inclusion() {
        let mut g = Graph::new(GraphId::from_u128(1));
        let a = NodeId::new();
        let b = NodeId::new();

        g.nodes.insert(
            a,
            node_at(
                CanvasPoint { x: 0.0, y: 0.0 },
                Some(CanvasSize {
                    width: 10.0,
                    height: 10.0,
                }),
            ),
        );
        g.nodes.insert(
            b,
            node_at(
                CanvasPoint { x: 9.0, y: 9.0 },
                Some(CanvasSize {
                    width: 5.0,
                    height: 5.0,
                }),
            ),
        );

        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&g);

        let rect = CanvasRect {
            origin: CanvasPoint { x: 0.0, y: 0.0 },
            size: CanvasSize {
                width: 10.0,
                height: 10.0,
            },
        };

        let partial = get_nodes_inside(
            &lookups,
            rect,
            GetNodesInsideOptions {
                inclusion: NodeInclusion::Partial,
                node_origin: (0.0, 0.0),
                include_hidden: true,
                fallback_size: None,
            },
        );
        let mut expected = vec![a, b];
        expected.sort();
        assert_eq!(partial, expected);

        let full = get_nodes_inside(
            &lookups,
            rect,
            GetNodesInsideOptions {
                inclusion: NodeInclusion::Full,
                node_origin: (0.0, 0.0),
                include_hidden: true,
                fallback_size: None,
            },
        );
        assert_eq!(full, vec![a]);
    }
}
