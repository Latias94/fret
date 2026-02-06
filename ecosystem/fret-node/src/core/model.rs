use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::TypeDesc;

use super::ids::{
    EdgeId, GraphId, GroupId, NodeId, NodeKindKey, PortId, PortKey, StickyNoteId, SymbolId,
};
use super::imports::GraphImport;

fn is_false(v: &bool) -> bool {
    !*v
}

/// Graph schema version (v1).
pub const GRAPH_VERSION: u32 = 1;

/// A 2D point in canvas space.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasPoint {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
}

/// A 2D size in canvas space.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasSize {
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// A rectangle in canvas space.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasRect {
    /// Top-left origin.
    pub origin: CanvasPoint,
    /// Size.
    pub size: CanvasSize,
}

/// Node graph document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Stable identity for editor-state lookup and cross-graph references.
    pub graph_id: GraphId,
    /// Schema version for migrations.
    pub graph_version: u32,

    /// Transitive graph dependencies (semantic subgraphs / libraries).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub imports: BTreeMap<GraphId, GraphImport>,

    /// Graph-scoped symbols (blackboard/variables).
    pub symbols: BTreeMap<SymbolId, Symbol>,

    /// Node instances.
    pub nodes: BTreeMap<NodeId, Node>,

    /// Port instances (owned by nodes, but stored in a flat map for stable lookup).
    pub ports: BTreeMap<PortId, Port>,

    /// Edges between ports.
    pub edges: BTreeMap<EdgeId, Edge>,

    /// Optional groups.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<GroupId, Group>,

    /// Optional sticky notes.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sticky_notes: BTreeMap<StickyNoteId, StickyNote>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new(GraphId::new())
    }
}

impl Graph {
    /// Creates a new, empty graph with the given id.
    pub fn new(graph_id: GraphId) -> Self {
        Self {
            graph_id,
            graph_version: GRAPH_VERSION,
            imports: BTreeMap::new(),
            symbols: BTreeMap::new(),
            nodes: BTreeMap::new(),
            ports: BTreeMap::new(),
            edges: BTreeMap::new(),
            groups: BTreeMap::new(),
            sticky_notes: BTreeMap::new(),
        }
    }
}

/// Node instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node kind identifier.
    pub kind: NodeKindKey,
    /// Node kind version (for per-kind migrations).
    pub kind_version: u32,
    /// Top-left position in canvas space.
    pub pos: CanvasPoint,

    /// Whether the node can be selected (XyFlow `node.selectable`).
    ///
    /// When omitted, the global `NodeGraphInteractionState.elements_selectable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selectable: Option<bool>,

    /// Whether the node can be dragged with pointer interactions (XyFlow `node.draggable`).
    ///
    /// When omitted, the global `NodeGraphInteractionState.nodes_draggable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draggable: Option<bool>,

    /// Whether the node can be used for creating connections via editor interactions (XyFlow
    /// `node.connectable`).
    ///
    /// When omitted, the global `NodeGraphInteractionState.nodes_connectable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connectable: Option<bool>,

    /// Whether the node can be deleted via editor interactions (XyFlow `node.deletable`).
    ///
    /// When omitted, the global `NodeGraphInteractionState.nodes_deletable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletable: Option<bool>,

    /// Optional group container id (subflow / parent frame).
    ///
    /// This is an editor-structure concept (XyFlow `parentId` mental model) and is intentionally
    /// orthogonal to semantic subgraphs (see ADR 0135).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<GroupId>,

    /// Optional per-node movement/resize extent override.
    ///
    /// This mirrors XyFlow's `node.extent` concept. It is an editor-structure constraint (UI-facing),
    /// not a semantic graph rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extent: Option<NodeExtent>,

    /// Whether moving/resizing this node can expand its parent container (if any).
    ///
    /// This mirrors XyFlow's `node.expandParent` behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expand_parent: Option<bool>,

    /// Optional explicit node size in logical px at zoom=1 (semantic sizing).
    ///
    /// The editor converts this into canvas space by dividing by the current zoom so node content
    /// remains readable under semantic zoom.
    ///
    /// When `None`, the editor derives the size from measured geometry or style defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<CanvasSize>,

    /// Whether the node is hidden (XyFlow `node.hidden`).
    ///
    /// Hidden nodes are excluded from derived geometry (hit-testing, rendering, fit-view).
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,

    /// Whether the node is collapsed.
    #[serde(default)]
    pub collapsed: bool,

    /// Stable port ordering for this node (UI-facing).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortId>,

    /// Opaque node payload (domain-owned).
    ///
    /// This must be preserved for unknown node kinds.
    #[serde(default)]
    pub data: Value,
}

/// Per-node movement/resize extent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeExtent {
    /// Constrain to the node's parent container (if any).
    Parent,
    /// Constrain to the given rect in canvas space.
    Rect { rect: CanvasRect },
}

/// Port direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    /// Input port.
    In,
    /// Output port.
    Out,
}

/// Port kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortKind {
    /// Data port.
    Data,
    /// Exec port (control flow).
    Exec,
}

/// Connection capacity for a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortCapacity {
    /// Single connection.
    Single,
    /// Multiple connections.
    Multi,
}

/// Port instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Owning node.
    pub node: NodeId,
    /// Schema key for stable migrations.
    pub key: PortKey,
    /// Port direction.
    pub dir: PortDirection,
    /// Port kind.
    pub kind: PortKind,
    /// Capacity rule.
    pub capacity: PortCapacity,

    /// Whether this port can be used for creating/accepting connections via editor interactions.
    ///
    /// This mirrors XyFlow handle-level `isConnectable`. When omitted, the owning node's
    /// `Node.connectable` / global `NodeGraphInteractionState.nodes_connectable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connectable: Option<bool>,

    /// Dictates whether a connection can start from this port.
    ///
    /// This mirrors XyFlow handle-level `isConnectableStart`. When omitted, the port is treated as
    /// start-connectable (subject to `connectable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connectable_start: Option<bool>,

    /// Dictates whether a connection can end on this port.
    ///
    /// This mirrors XyFlow handle-level `isConnectableEnd`. When omitted, the port is treated as
    /// end-connectable (subject to `connectable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connectable_end: Option<bool>,

    /// Optional type descriptor.
    ///
    /// Profiles may choose to infer or override this via concretization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<TypeDesc>,

    /// Opaque port payload (domain-owned).
    #[serde(default)]
    pub data: Value,
}

/// Edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Typed data flow.
    Data,
    /// Exec/control flow.
    Exec,
}

/// Edge between two ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Edge kind.
    pub kind: EdgeKind,
    /// Source port.
    pub from: PortId,
    /// Target port.
    pub to: PortId,
    /// Whether the edge can be selected (XyFlow `edge.selectable`).
    ///
    /// When omitted, the global `NodeGraphInteractionState.edges_selectable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selectable: Option<bool>,

    /// Whether the edge can be deleted via editor interactions (XyFlow `edge.deletable`).
    ///
    /// When omitted, the global `NodeGraphInteractionState.edges_deletable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletable: Option<bool>,

    /// Whether the edge can be reconnected via editor interactions (XyFlow `edge.reconnectable`).
    ///
    /// In XyFlow this field is a `boolean | 'source' | 'target'`. `true` enables reconnecting both
    /// endpoints, `'source'` only enables reconnecting the source endpoint and `'target'` only
    /// enables reconnecting the target endpoint.
    ///
    /// When omitted, the global `NodeGraphInteractionState.edges_reconnectable` decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconnectable: Option<EdgeReconnectable>,
}

/// Per-edge reconnect enablement (XyFlow `edge.reconnectable`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EdgeReconnectable {
    Bool(bool),
    Endpoint(EdgeReconnectableEndpoint),
}

/// Which endpoint is reconnectable (`'source' | 'target'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeReconnectableEndpoint {
    Source,
    Target,
}

/// Graph-scoped symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Display name.
    pub name: String,
    /// Type descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<TypeDesc>,
    /// Default value (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
    /// Arbitrary domain metadata.
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub meta: Value,
}

/// A node group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Display name.
    pub title: String,
    /// Group bounds in canvas space.
    pub rect: CanvasRect,
    /// Group color (domain/theme-owned).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// A sticky note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickyNote {
    /// Markdown/plain text body.
    pub text: String,
    /// Note bounds in canvas space.
    pub rect: CanvasRect,
    /// Note color (domain/theme-owned).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}
