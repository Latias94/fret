//! Deterministic graph fragments for clipboard, duplication, and merges.
//!
//! A fragment is a self-contained subset of a graph that can be serialized and pasted into another
//! graph by remapping IDs.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json as json;
use uuid::Uuid;

use crate::core::{
    CanvasPoint, Edge, EdgeId, Graph, Group, GroupId, Node, NodeId, Port, PortId, StickyNote,
    StickyNoteId, Symbol, SymbolId, symbol_ref_node_data, symbol_ref_target_symbol_id,
};
use crate::ops::{GraphOp, GraphTransaction};

/// Wrapper version for `GraphFragment`.
pub const GRAPH_FRAGMENT_VERSION: u32 = 1;

/// Clipboard header for `GraphFragment` payloads.
pub const GRAPH_FRAGMENT_CLIPBOARD_PREFIX: &str = "fret-node.fragment.v1\n";

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
    /// Groups/notes are not inferred; callers may add them explicitly.
    ///
    /// Symbols are inferred for built-in symbol-ref nodes (`core::SYMBOL_REF_NODE_KIND`) so
    /// copy/paste can remain self-contained for the "blackboard variables" contract.
    pub fn from_nodes(graph: &Graph, nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Self::from_selection(graph, nodes, std::iter::empty())
    }

    /// Builds a fragment from a selection of nodes and groups.
    ///
    /// Captures:
    /// - selected groups,
    /// - selected nodes,
    /// - nodes inside selected groups,
    /// - ports for all captured nodes,
    /// - edges that connect between captured nodes.
    ///
    /// Nodes are detached from their parent group unless that group is included in the fragment.
    pub fn from_selection(
        graph: &Graph,
        selected_nodes: impl IntoIterator<Item = NodeId>,
        selected_groups: impl IntoIterator<Item = GroupId>,
    ) -> Self {
        let mut out = GraphFragment::default();

        let groups: BTreeSet<GroupId> = selected_groups.into_iter().collect();
        for group_id in &groups {
            if let Some(group) = graph.groups.get(group_id) {
                out.groups.insert(*group_id, group.clone());
            }
        }

        let mut nodes: BTreeSet<NodeId> = selected_nodes.into_iter().collect();
        if !groups.is_empty() {
            for (node_id, node) in &graph.nodes {
                if node.parent.is_some_and(|p| groups.contains(&p)) {
                    nodes.insert(*node_id);
                }
            }
        }

        for node_id in &nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let mut node = node.clone();
                if node.parent.is_some_and(|p| !groups.contains(&p)) {
                    node.parent = None;
                }
                out.nodes.insert(*node_id, node);
            }
        }

        for (node_id, node) in &out.nodes {
            let Ok(Some(symbol_id)) = symbol_ref_target_symbol_id(*node_id, node) else {
                continue;
            };
            if let Some(symbol) = graph.symbols.get(&symbol_id) {
                out.symbols.insert(symbol_id, symbol.clone());
            }
        }

        for (port_id, port) in &graph.ports {
            if nodes.contains(&port.node) {
                out.ports.insert(*port_id, port.clone());
            }
        }

        for (edge_id, edge) in &graph.edges {
            let from_node = graph.ports.get(&edge.from).map(|p| p.node);
            let to_node = graph.ports.get(&edge.to).map(|p| p.node);
            if let (Some(from_node), Some(to_node)) = (from_node, to_node)
                && nodes.contains(&from_node)
                && nodes.contains(&to_node)
            {
                out.edges.insert(*edge_id, edge.clone());
            }
        }

        out
    }

    /// Serializes this fragment to a clipboard-friendly text payload.
    pub fn to_clipboard_text(&self) -> Result<String, json::Error> {
        let json = json::to_string(self)?;
        Ok(format!("{GRAPH_FRAGMENT_CLIPBOARD_PREFIX}{json}"))
    }

    /// Parses a fragment from clipboard text.
    ///
    /// Accepts both:
    /// - the canonical `fret-node.fragment.v1` header format, and
    /// - raw JSON (useful for debugging and external tooling).
    pub fn from_clipboard_text(text: &str) -> Result<Self, json::Error> {
        let payload = text
            .strip_prefix(GRAPH_FRAGMENT_CLIPBOARD_PREFIX)
            .unwrap_or(text);
        json::from_str(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::GraphFragment;
    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, Group, GroupId, Node,
        NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use serde_json::Value;

    fn node(kind: &str, pos: (f32, f32), parent: Option<GroupId>, ports: &[PortId]) -> Node {
        Node {
            kind: NodeKindKey::new(kind),
            kind_version: 1,
            pos: CanvasPoint { x: pos.0, y: pos.1 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: ports.to_vec(),
            data: Value::Null,
        }
    }

    fn port(node: NodeId, key: &str, dir: PortDirection) -> Port {
        Port {
            node,
            key: PortKey::new(key),
            dir,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        }
    }

    #[test]
    fn from_selection_detaches_nodes_when_group_is_not_selected() {
        let mut graph = Graph::default();

        let group_id = GroupId::new();
        graph.groups.insert(
            group_id,
            Group {
                title: "G".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 100.0,
                    },
                },
                color: None,
            },
        );

        let node_id = NodeId::new();
        let in_port = PortId::new();
        graph
            .nodes
            .insert(node_id, node("A", (1.0, 2.0), Some(group_id), &[in_port]));
        graph
            .ports
            .insert(in_port, port(node_id, "in", PortDirection::In));

        let fragment = GraphFragment::from_selection(&graph, [node_id], std::iter::empty());
        assert!(fragment.groups.is_empty());
        assert_eq!(fragment.nodes.len(), 1);
        assert_eq!(fragment.nodes[&node_id].parent, None);
    }

    #[test]
    fn from_selection_keeps_parent_when_group_is_selected_and_includes_children() {
        let mut graph = Graph::default();

        let group_id = GroupId::new();
        graph.groups.insert(
            group_id,
            Group {
                title: "G".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 100.0,
                    },
                },
                color: None,
            },
        );

        let a = NodeId::new();
        let a_out = PortId::new();
        graph
            .nodes
            .insert(a, node("A", (1.0, 2.0), Some(group_id), &[a_out]));
        graph
            .ports
            .insert(a_out, port(a, "out", PortDirection::Out));

        let b = NodeId::new();
        let b_in = PortId::new();
        graph
            .nodes
            .insert(b, node("B", (3.0, 4.0), Some(group_id), &[b_in]));
        graph.ports.insert(b_in, port(b, "in", PortDirection::In));

        let e = EdgeId::new();
        graph.edges.insert(
            e,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let fragment = GraphFragment::from_selection(&graph, std::iter::empty(), [group_id]);
        assert_eq!(fragment.groups.len(), 1);
        assert!(fragment.nodes.contains_key(&a));
        assert!(fragment.nodes.contains_key(&b));
        assert_eq!(fragment.nodes[&a].parent, Some(group_id));
        assert_eq!(fragment.nodes[&b].parent, Some(group_id));
        assert_eq!(fragment.edges.len(), 1);
    }

    #[test]
    fn clipboard_text_roundtrips_with_prefix() {
        let fragment = GraphFragment::default();
        let text = fragment.to_clipboard_text().expect("serialize");
        assert!(text.starts_with(super::GRAPH_FRAGMENT_CLIPBOARD_PREFIX));
        let parsed = GraphFragment::from_clipboard_text(&text).expect("parse");
        assert_eq!(parsed.version, fragment.version);
        assert_eq!(parsed.nodes.len(), 0);
    }

    #[test]
    fn clipboard_text_accepts_raw_json() {
        let fragment = GraphFragment::default();
        let json = serde_json::to_string(&fragment).expect("serialize");
        let parsed = GraphFragment::from_clipboard_text(&json).expect("parse");
        assert_eq!(parsed.version, fragment.version);
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

            if let Ok(Some(old_symbol_id)) = symbol_ref_target_symbol_id(*old_id, old_node)
                && let Some(new_symbol_id) = symbol_map.get(&old_symbol_id)
            {
                node.data = symbol_ref_node_data(*new_symbol_id);
            }

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
                selectable: old_edge.selectable,
                deletable: old_edge.deletable,
                reconnectable: old_edge.reconnectable,
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
