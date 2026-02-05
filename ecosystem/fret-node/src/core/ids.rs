use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable identifier for a graph document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphId(pub Uuid);

impl GraphId {
    /// Generates a new random graph id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a graph id from a stable 128-bit value.
    pub fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }

    /// Creates a graph id from raw UUID bytes.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes(bytes))
    }
}

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Stable identifier for a node instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub Uuid);

impl NodeId {
    /// Generates a new random node id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Stable identifier for a port instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortId(pub Uuid);

impl PortId {
    /// Generates a new random port id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Stable identifier for an edge instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EdgeId(pub Uuid);

impl EdgeId {
    /// Generates a new random edge id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an edge id from a stable 128-bit value.
    pub fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }
}

/// Stable identifier for a graph-scoped symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolId(pub Uuid);

impl SymbolId {
    /// Generates a new random symbol id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a symbol id from a stable 128-bit value.
    pub fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }
}

/// Stable identifier for a group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub Uuid);

impl GroupId {
    /// Generates a new random group id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Stable identifier for a sticky note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StickyNoteId(pub Uuid);

impl StickyNoteId {
    /// Generates a new random sticky note id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Stable identifier for a node kind.
///
/// This is a namespaced string identifier (e.g. `core.math.add`, `plugin.acme.http_request`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeKindKey(pub String);

impl NodeKindKey {
    /// Creates a new node kind key.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

/// Stable identifier for a schema-declared port.
///
/// `PortKey` must remain stable across versions of a node kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortKey(pub String);

impl PortKey {
    /// Creates a new port key.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}
