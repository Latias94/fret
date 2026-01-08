//! Deterministic graph fragments for clipboard, duplication, and merges.
//!
//! A fragment is a self-contained subset of a graph that can be serialized and pasted into another
//! graph by remapping IDs.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::{
    CanvasPoint, Edge, EdgeId, Graph, Group, GroupId, Node, NodeId, Port, PortId, StickyNote,
    StickyNoteId, Symbol, SymbolId,
};
use crate::ops::{GraphOp, GraphTransaction};

/// Wrapper version for `GraphFragment`.
pub const GRAPH_FRAGMENT_VERSION: u32 = 1;

/// A deterministic, serializable graph fragment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFragment {
    pub version: u32,
    pub nodes: BTreeMap<NodeId, Node>,
    pub ports: BTreeMap<PortId, Port>,
    pub edges: BTreeMap<EdgeId, Edge>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<GroupId, Group>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sticky_notes: BTreeMap<StickyNoteId, StickyNote>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub symbols: BTreeMap<SymbolId, Symbol>,
}

impl Default for GraphFragment {
    fn default() -> Self {
        Self {
            version: GRAPH_FRAGMENT_VERSION,
            nodes: BTreeMap::new(),
            ports: BTreeMap::new(),
            edges: BTreeMap::new(),
            groups: BTreeMap::new(),
            sticky_notes: BTreeMap::new(),
            symbols: BTreeMap::new(),
        }
    }
}

impl GraphFragment {
    /// Builds a fragment from a set of nodes, capturing:
    /// - the selected nodes,
    /// - their ports,
    /// - edges that connect between selected nodes.
    ///
    /// Groups/notes/symbols are currently not inferred; callers may add them explicitly.
    pub fn from_nodes(graph: &Graph, nodes: impl IntoIterator<Item = NodeId>) -> Self {
        let mut out = GraphFragment::default();

        let selected: BTreeSet<NodeId> = nodes.into_iter().collect();
        for node_id in &selected {
            if let Some(node) = graph.nodes.get(node_id) {
                let mut node = node.clone();
                // Groups are currently not inferred when building fragments, so detach copied nodes
                // from their container by default.
                node.parent = None;
                out.nodes.insert(*node_id, node);
            }
        }

        for (port_id, port) in &graph.ports {
            if selected.contains(&port.node) {
                out.ports.insert(*port_id, port.clone());
            }
        }

        for (edge_id, edge) in &graph.edges {
            let from_node = graph.ports.get(&edge.from).map(|p| p.node);
            let to_node = graph.ports.get(&edge.to).map(|p| p.node);
            if let (Some(from_node), Some(to_node)) = (from_node, to_node) {
                if selected.contains(&from_node) && selected.contains(&to_node) {
                    out.edges.insert(*edge_id, edge.clone());
                }
            }
        }

        out
    }
}

/// Seed used for deterministic ID remapping.
///
/// Using different seeds yields different IDs while keeping determinism for a given paste action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IdRemapSeed(pub Uuid);

impl IdRemapSeed {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Deterministic ID remapper (UUID v5 under a caller-provided seed).
#[derive(Debug, Clone)]
pub struct IdRemapper {
    seed: IdRemapSeed,
}

impl IdRemapper {
    pub fn new(seed: IdRemapSeed) -> Self {
        Self { seed }
    }

    fn remap_uuid(&self, old: Uuid) -> Uuid {
        // Use UUID v5 in a deterministic namespace to avoid collisions across elements.
        // The caller should pick a new seed for each paste/duplicate action.
        Uuid::new_v5(&self.seed.0, old.as_bytes())
    }

    pub fn remap_node(&self, id: NodeId) -> NodeId {
        NodeId(self.remap_uuid(id.0))
    }

    pub fn remap_port(&self, id: PortId) -> PortId {
        PortId(self.remap_uuid(id.0))
    }

    pub fn remap_edge(&self, id: EdgeId) -> EdgeId {
        EdgeId(self.remap_uuid(id.0))
    }

    pub fn remap_group(&self, id: GroupId) -> GroupId {
        GroupId(self.remap_uuid(id.0))
    }

    pub fn remap_note(&self, id: StickyNoteId) -> StickyNoteId {
        StickyNoteId(self.remap_uuid(id.0))
    }

    pub fn remap_symbol(&self, id: SymbolId) -> SymbolId {
        SymbolId(self.remap_uuid(id.0))
    }
}

/// Paste tuning for translating fragments into a destination graph.
#[derive(Debug, Clone, Copy)]
pub struct PasteTuning {
    /// Additional offset applied to every pasted node position.
    pub offset: CanvasPoint,
}

impl Default for PasteTuning {
    fn default() -> Self {
        Self {
            offset: CanvasPoint { x: 0.0, y: 0.0 },
        }
    }
}

impl GraphFragment {
    /// Remaps IDs and produces a transaction that inserts the fragment into a graph.
    ///
    /// The resulting transaction is deterministic for a given seed.
    pub fn to_paste_transaction(
        &self,
        remapper: &IdRemapper,
        tuning: PasteTuning,
    ) -> GraphTransaction {
        let mut tx = GraphTransaction::new();

        let mut group_map: BTreeMap<GroupId, GroupId> = BTreeMap::new();
        for group_id in self.groups.keys() {
            group_map.insert(*group_id, remapper.remap_group(*group_id));
        }

        let mut node_map: BTreeMap<NodeId, NodeId> = BTreeMap::new();
        for node_id in self.nodes.keys() {
            node_map.insert(*node_id, remapper.remap_node(*node_id));
        }

        let mut port_map: BTreeMap<PortId, PortId> = BTreeMap::new();
        for port_id in self.ports.keys() {
            port_map.insert(*port_id, remapper.remap_port(*port_id));
        }

        let mut symbol_map: BTreeMap<SymbolId, SymbolId> = BTreeMap::new();
        for symbol_id in self.symbols.keys() {
            symbol_map.insert(*symbol_id, remapper.remap_symbol(*symbol_id));
        }

        for (old_id, old_symbol) in &self.symbols {
            let new_id = symbol_map[old_id];
            tx.push(GraphOp::AddSymbol {
                id: new_id,
                symbol: old_symbol.clone(),
            });
        }

        for (old_id, group) in &self.groups {
            tx.push(GraphOp::AddGroup {
                id: group_map[old_id],
                group: group.clone(),
            });
        }

        for (old_id, old_node) in &self.nodes {
            let new_id = node_map[old_id];
            let mut node = old_node.clone();
            node.pos = CanvasPoint {
                x: node.pos.x + tuning.offset.x,
                y: node.pos.y + tuning.offset.y,
            };
            node.parent = node
                .parent
                .and_then(|old_parent| group_map.get(&old_parent).copied());
            // Port ordering is remapped after ports are added.
            node.ports = Vec::new();
            tx.push(GraphOp::AddNode { id: new_id, node });
        }

        for (old_port_id, old_port) in &self.ports {
            let new_port_id = port_map[old_port_id];
            let mut port = old_port.clone();
            port.node = node_map[&port.node];
            tx.push(GraphOp::AddPort {
                id: new_port_id,
                port,
            });
        }

        // Restore node port ordering deterministically.
        for (old_node_id, old_node) in &self.nodes {
            let new_node_id = node_map[old_node_id];
            let mut ports: Vec<PortId> = Vec::new();
            for old_port_id in &old_node.ports {
                if let Some(new_port) = port_map.get(old_port_id) {
                    ports.push(*new_port);
                }
            }
            if !ports.is_empty() {
                tx.push(GraphOp::SetNodePorts {
                    id: new_node_id,
                    from: Vec::new(),
                    to: ports,
                });
            }
        }

        for (old_edge_id, old_edge) in &self.edges {
            let new_edge_id = remapper.remap_edge(*old_edge_id);
            let edge = Edge {
                kind: old_edge.kind,
                from: port_map[&old_edge.from],
                to: port_map[&old_edge.to],
            };
            tx.push(GraphOp::AddEdge {
                id: new_edge_id,
                edge,
            });
        }

        for (old_id, note) in &self.sticky_notes {
            tx.push(GraphOp::AddStickyNote {
                id: remapper.remap_note(*old_id),
                note: note.clone(),
            });
        }

        tx
    }
}
