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
    pub collapsed: bool,
    pub ports: Vec<PortId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeLookupEntry {
    pub kind: EdgeKind,
    pub from: PortId,
    pub to: PortId,
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
                    collapsed: node.collapsed,
                    ports: node.ports.clone(),
                },
            );
        }

        for (id, edge) in &graph.edges {
            self.edge_lookup.insert(
                *id,
                EdgeLookupEntry {
                    kind: edge.kind,
                    from: edge.from,
                    to: edge.to,
                    reconnectable: edge.reconnectable,
                },
            );

            let Some(from_port) = graph.ports.get(&edge.from) else {
                continue;
            };
            let Some(to_port) = graph.ports.get(&edge.to) else {
                continue;
            };

            let source_node = from_port.node;
            let target_node = to_port.node;

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

    pub fn connections(
        &self,
        key: ConnectionLookupKey,
    ) -> Option<&HashMap<EdgeId, HandleConnection>> {
        self.connection_lookup.get(&key)
    }

    pub fn connections_for_node(&self, node: NodeId) -> Option<&HashMap<EdgeId, HandleConnection>> {
        self.connections(ConnectionLookupKey::Node(node))
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
            .or_insert_with(HashMap::new)
            .insert(conn.edge, conn);
    }
}
