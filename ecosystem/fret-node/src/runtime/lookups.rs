//! Canonical lookup maps for fast graph queries (XyFlow-style).
//!
//! XyFlow maintains several "lookup maps" alongside its canonical node/edge arrays:
//! - `nodeLookup` (id -> internal node)
//! - `edgeLookup` (id -> edge)
//! - `connectionLookup` (node/handle -> connections)
//!
//! In fret-node the serialized document (`core::Graph`) is already map-based, but a first-class,
//! headless-safe lookup surface is still useful for:
//! - consistent adjacency queries (node/port -> incident edges),
//! - avoiding repeated full scans in editor shells,
//! - providing a stable substrate for B-layer tooling and middleware.

use std::collections::HashMap;

use crate::core::{
    CanvasPoint, CanvasSize, EdgeId, EdgeKind, EdgeReconnectable, Graph, GroupId, NodeId,
    NodeKindKey, PortDirection, PortId,
};
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionSide {
    Source,
    Target,
}

impl ConnectionSide {
    pub fn from_port_dir(dir: PortDirection) -> Self {
        match dir {
            PortDirection::In => Self::Target,
            PortDirection::Out => Self::Source,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionLookupKey {
    Node(NodeId),
    NodeSide {
        node: NodeId,
        side: ConnectionSide,
    },
    NodeSidePort {
        node: NodeId,
        side: ConnectionSide,
        port: PortId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleConnection {
    pub edge: EdgeId,
    pub source_node: NodeId,
    pub source_port: PortId,
    pub target_node: NodeId,
    pub target_port: PortId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeLookupEntry {
    pub kind: NodeKindKey,
    pub kind_version: u32,
    pub pos: CanvasPoint,
    pub parent: Option<GroupId>,
    pub size: Option<CanvasSize>,
    pub hidden: bool,
    pub collapsed: bool,
    pub ports: Vec<PortId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeLookupEntry {
    pub kind: EdgeKind,
    pub from: PortId,
    pub to: PortId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub reconnectable: Option<EdgeReconnectable>,
}

#[derive(Debug, Default)]
pub struct NodeGraphLookups {
    pub node_lookup: HashMap<NodeId, NodeLookupEntry>,
    pub edge_lookup: HashMap<EdgeId, EdgeLookupEntry>,
    pub connection_lookup: HashMap<ConnectionLookupKey, HashMap<EdgeId, HandleConnection>>,
}

impl NodeGraphLookups {
    pub fn rebuild_from(&mut self, graph: &Graph) {
        self.node_lookup.clear();
        self.edge_lookup.clear();
        self.connection_lookup.clear();

        for (id, node) in &graph.nodes {
            self.node_lookup.insert(
                *id,
                NodeLookupEntry {
                    kind: node.kind.clone(),
                    kind_version: node.kind_version,
                    pos: node.pos,
                    parent: node.parent,
                    size: node.size,
                    hidden: node.hidden,
                    collapsed: node.collapsed,
                    ports: node.ports.clone(),
                },
            );
        }

        for (id, edge) in &graph.edges {
            let Some(from_port) = graph.ports.get(&edge.from) else {
                continue;
            };
            let Some(to_port) = graph.ports.get(&edge.to) else {
                continue;
            };

            let source_node = from_port.node;
            let target_node = to_port.node;

            self.edge_lookup.insert(
                *id,
                EdgeLookupEntry {
                    kind: edge.kind,
                    from: edge.from,
                    to: edge.to,
                    from_node: source_node,
                    to_node: target_node,
                    reconnectable: edge.reconnectable,
                },
            );

            let conn = HandleConnection {
                edge: *id,
                source_node,
                source_port: edge.from,
                target_node,
                target_port: edge.to,
                kind: edge.kind,
            };

            self.add_connection(ConnectionLookupKey::Node(source_node), conn);
            self.add_connection(
                ConnectionLookupKey::NodeSide {
                    node: source_node,
                    side: ConnectionSide::Source,
                },
                conn,
            );
            self.add_connection(
                ConnectionLookupKey::NodeSidePort {
                    node: source_node,
                    side: ConnectionSide::Source,
                    port: edge.from,
                },
                conn,
            );

            self.add_connection(ConnectionLookupKey::Node(target_node), conn);
            self.add_connection(
                ConnectionLookupKey::NodeSide {
                    node: target_node,
                    side: ConnectionSide::Target,
                },
                conn,
            );
            self.add_connection(
                ConnectionLookupKey::NodeSidePort {
                    node: target_node,
                    side: ConnectionSide::Target,
                    port: edge.to,
                },
                conn,
            );
        }
    }

    pub fn apply_transaction(&mut self, graph: &Graph, tx: &GraphTransaction) {
        for op in &tx.ops {
            if !self.apply_op(graph, op) {
                self.rebuild_from(graph);
                return;
            }
        }
    }

    pub fn connections(
        &self,
        key: ConnectionLookupKey,
    ) -> Option<&HashMap<EdgeId, HandleConnection>> {
        self.connection_lookup.get(&key)
    }

    pub fn connections_for_node(&self, node: NodeId) -> Option<&HashMap<EdgeId, HandleConnection>> {
        self.connections(ConnectionLookupKey::Node(node))
    }

    pub fn connections_for_node_side(
        &self,
        node: NodeId,
        side: ConnectionSide,
    ) -> Option<&HashMap<EdgeId, HandleConnection>> {
        self.connections(ConnectionLookupKey::NodeSide { node, side })
    }

    pub fn connections_for_port(
        &self,
        node: NodeId,
        side: ConnectionSide,
        port: PortId,
    ) -> Option<&HashMap<EdgeId, HandleConnection>> {
        self.connections(ConnectionLookupKey::NodeSidePort { node, side, port })
    }

    fn add_connection(&mut self, key: ConnectionLookupKey, conn: HandleConnection) {
        self.connection_lookup
            .entry(key)
            .or_default()
            .insert(conn.edge, conn);
    }

    fn remove_connection(&mut self, key: ConnectionLookupKey, edge: EdgeId) {
        let Some(map) = self.connection_lookup.get_mut(&key) else {
            return;
        };
        map.remove(&edge);
        if map.is_empty() {
            self.connection_lookup.remove(&key);
        }
    }

    fn add_edge_connection(&mut self, entry: HandleConnection) {
        self.add_connection(ConnectionLookupKey::Node(entry.source_node), entry);
        self.add_connection(
            ConnectionLookupKey::NodeSide {
                node: entry.source_node,
                side: ConnectionSide::Source,
            },
            entry,
        );
        self.add_connection(
            ConnectionLookupKey::NodeSidePort {
                node: entry.source_node,
                side: ConnectionSide::Source,
                port: entry.source_port,
            },
            entry,
        );

        self.add_connection(ConnectionLookupKey::Node(entry.target_node), entry);
        self.add_connection(
            ConnectionLookupKey::NodeSide {
                node: entry.target_node,
                side: ConnectionSide::Target,
            },
            entry,
        );
        self.add_connection(
            ConnectionLookupKey::NodeSidePort {
                node: entry.target_node,
                side: ConnectionSide::Target,
                port: entry.target_port,
            },
            entry,
        );
    }

    fn remove_edge_connection(&mut self, entry: HandleConnection) {
        self.remove_connection(ConnectionLookupKey::Node(entry.source_node), entry.edge);
        self.remove_connection(
            ConnectionLookupKey::NodeSide {
                node: entry.source_node,
                side: ConnectionSide::Source,
            },
            entry.edge,
        );
        self.remove_connection(
            ConnectionLookupKey::NodeSidePort {
                node: entry.source_node,
                side: ConnectionSide::Source,
                port: entry.source_port,
            },
            entry.edge,
        );

        self.remove_connection(ConnectionLookupKey::Node(entry.target_node), entry.edge);
        self.remove_connection(
            ConnectionLookupKey::NodeSide {
                node: entry.target_node,
                side: ConnectionSide::Target,
            },
            entry.edge,
        );
        self.remove_connection(
            ConnectionLookupKey::NodeSidePort {
                node: entry.target_node,
                side: ConnectionSide::Target,
                port: entry.target_port,
            },
            entry.edge,
        );
    }

    fn slow_remove_edge_from_connection_lookup(&mut self, edge: EdgeId) {
        let mut empty: Vec<ConnectionLookupKey> = Vec::new();
        for (key, map) in &mut self.connection_lookup {
            map.remove(&edge);
            if map.is_empty() {
                empty.push(*key);
            }
        }
        for k in empty {
            self.connection_lookup.remove(&k);
        }
    }

    fn connection_from_edge_lookup(&self, edge: EdgeId) -> Option<HandleConnection> {
        let e = self.edge_lookup.get(&edge)?;
        Some(HandleConnection {
            edge,
            source_node: e.from_node,
            source_port: e.from,
            target_node: e.to_node,
            target_port: e.to,
            kind: e.kind,
        })
    }

    fn edge_lookup_entry_from_graph(
        graph: &Graph,
        id: EdgeId,
        kind: EdgeKind,
        endpoints: EdgeEndpoints,
        reconnectable: Option<EdgeReconnectable>,
    ) -> Option<(EdgeLookupEntry, HandleConnection)> {
        let from_port = graph.ports.get(&endpoints.from)?;
        let to_port = graph.ports.get(&endpoints.to)?;
        let source_node = from_port.node;
        let target_node = to_port.node;
        let entry = EdgeLookupEntry {
            kind,
            from: endpoints.from,
            to: endpoints.to,
            from_node: source_node,
            to_node: target_node,
            reconnectable,
        };
        let conn = HandleConnection {
            edge: id,
            source_node,
            source_port: endpoints.from,
            target_node,
            target_port: endpoints.to,
            kind,
        };
        Some((entry, conn))
    }

    fn apply_op(&mut self, graph: &Graph, op: &GraphOp) -> bool {
        match op {
            GraphOp::AddNode { id, node } => {
                self.node_lookup.insert(
                    *id,
                    NodeLookupEntry {
                        kind: node.kind.clone(),
                        kind_version: node.kind_version,
                        pos: node.pos,
                        parent: node.parent,
                        size: node.size,
                        hidden: node.hidden,
                        collapsed: node.collapsed,
                        ports: node.ports.clone(),
                    },
                );
                true
            }
            GraphOp::RemoveNode { id, edges, .. } => {
                for (edge_id, _edge) in edges {
                    if let Some(conn) = self.connection_from_edge_lookup(*edge_id) {
                        self.remove_edge_connection(conn);
                    } else {
                        self.slow_remove_edge_from_connection_lookup(*edge_id);
                    }
                    self.edge_lookup.remove(edge_id);
                }
                self.node_lookup.remove(id);
                true
            }
            GraphOp::SetNodePos { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.pos = *to;
                    return true;
                }
                if let Some(node) = graph.nodes.get(id) {
                    self.node_lookup.insert(
                        *id,
                        NodeLookupEntry {
                            kind: node.kind.clone(),
                            kind_version: node.kind_version,
                            pos: node.pos,
                            parent: node.parent,
                            size: node.size,
                            hidden: node.hidden,
                            collapsed: node.collapsed,
                            ports: node.ports.clone(),
                        },
                    );
                    return true;
                }
                false
            }
            GraphOp::SetNodeKind { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.kind = to.clone();
                    return true;
                }
                if let Some(node) = graph.nodes.get(id) {
                    self.node_lookup.insert(
                        *id,
                        NodeLookupEntry {
                            kind: node.kind.clone(),
                            kind_version: node.kind_version,
                            pos: node.pos,
                            parent: node.parent,
                            size: node.size,
                            hidden: node.hidden,
                            collapsed: node.collapsed,
                            ports: node.ports.clone(),
                        },
                    );
                    return true;
                }
                false
            }
            GraphOp::SetNodeKindVersion { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.kind_version = *to;
                    return true;
                }
                if let Some(node) = graph.nodes.get(id) {
                    self.node_lookup.insert(
                        *id,
                        NodeLookupEntry {
                            kind: node.kind.clone(),
                            kind_version: node.kind_version,
                            pos: node.pos,
                            parent: node.parent,
                            size: node.size,
                            hidden: node.hidden,
                            collapsed: node.collapsed,
                            ports: node.ports.clone(),
                        },
                    );
                    return true;
                }
                false
            }
            GraphOp::SetNodeParent { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.parent = *to;
                    return true;
                }
                false
            }
            GraphOp::SetNodeSize { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.size = *to;
                    return true;
                }
                false
            }
            GraphOp::SetNodeCollapsed { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.collapsed = *to;
                    return true;
                }
                false
            }
            GraphOp::SetNodePorts { id, to, .. } => {
                if let Some(n) = self.node_lookup.get_mut(id) {
                    n.ports = to.clone();
                    return true;
                }
                false
            }
            GraphOp::RemovePort { edges, .. } => {
                for (edge_id, _edge) in edges {
                    if let Some(conn) = self.connection_from_edge_lookup(*edge_id) {
                        self.remove_edge_connection(conn);
                    } else {
                        self.slow_remove_edge_from_connection_lookup(*edge_id);
                    }
                    self.edge_lookup.remove(edge_id);
                }
                true
            }
            GraphOp::AddEdge { id, edge } => {
                let endpoints = EdgeEndpoints {
                    from: edge.from,
                    to: edge.to,
                };
                let Some((entry, conn)) = Self::edge_lookup_entry_from_graph(
                    graph,
                    *id,
                    edge.kind,
                    endpoints,
                    edge.reconnectable,
                ) else {
                    return false;
                };
                self.edge_lookup.insert(*id, entry);
                self.add_edge_connection(conn);
                true
            }
            GraphOp::RemoveEdge { id, .. } => {
                if let Some(conn) = self.connection_from_edge_lookup(*id) {
                    self.remove_edge_connection(conn);
                } else {
                    self.slow_remove_edge_from_connection_lookup(*id);
                }
                self.edge_lookup.remove(id);
                true
            }
            GraphOp::SetEdgeKind { id, to, .. } => {
                if let Some(e) = self.edge_lookup.get_mut(id) {
                    e.kind = *to;
                }
                let Some(conn) = self.connection_from_edge_lookup(*id) else {
                    self.slow_update_edge_kind_in_connection_lookup(*id, *to);
                    return true;
                };
                self.update_edge_kind_in_connection_lookup(conn, *to);
                true
            }
            GraphOp::SetEdgeEndpoints { id, from, to } => {
                if let Some(prev) = self.edge_lookup.get(id).copied() {
                    self.remove_edge_connection(HandleConnection {
                        edge: *id,
                        source_node: prev.from_node,
                        source_port: prev.from,
                        target_node: prev.to_node,
                        target_port: prev.to,
                        kind: prev.kind,
                    });
                } else {
                    // try best-effort removal based on old edge id
                    self.slow_remove_edge_from_connection_lookup(*id);
                }

                let kind = graph.edges.get(id).map(|e| e.kind).unwrap_or_else(|| {
                    self.edge_lookup
                        .get(id)
                        .map(|e| e.kind)
                        .unwrap_or(EdgeKind::Data)
                });
                let reconnectable = graph
                    .edges
                    .get(id)
                    .and_then(|e| e.reconnectable)
                    .or_else(|| self.edge_lookup.get(id).and_then(|e| e.reconnectable));

                let Some((entry, conn)) =
                    Self::edge_lookup_entry_from_graph(graph, *id, kind, *to, reconnectable)
                else {
                    // revert to full rebuild if we cannot compute endpoint owners
                    return false;
                };
                self.edge_lookup.insert(*id, entry);
                self.add_edge_connection(conn);
                let _ = from;
                true
            }
            _ => true,
        }
    }

    fn update_edge_kind_in_connection_lookup(&mut self, conn: HandleConnection, kind: EdgeKind) {
        for key in [
            ConnectionLookupKey::Node(conn.source_node),
            ConnectionLookupKey::NodeSide {
                node: conn.source_node,
                side: ConnectionSide::Source,
            },
            ConnectionLookupKey::NodeSidePort {
                node: conn.source_node,
                side: ConnectionSide::Source,
                port: conn.source_port,
            },
            ConnectionLookupKey::Node(conn.target_node),
            ConnectionLookupKey::NodeSide {
                node: conn.target_node,
                side: ConnectionSide::Target,
            },
            ConnectionLookupKey::NodeSidePort {
                node: conn.target_node,
                side: ConnectionSide::Target,
                port: conn.target_port,
            },
        ] {
            if let Some(map) = self.connection_lookup.get_mut(&key)
                && let Some(entry) = map.get_mut(&conn.edge)
            {
                entry.kind = kind;
            }
        }
    }

    fn slow_update_edge_kind_in_connection_lookup(&mut self, edge: EdgeId, kind: EdgeKind) {
        for map in self.connection_lookup.values_mut() {
            if let Some(entry) = map.get_mut(&edge) {
                entry.kind = kind;
            }
        }
    }
}
