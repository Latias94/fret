//! Node and port schema registry.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{NodeKindKey, PortCapacity, PortDirection, PortKey, PortKind};
use crate::types::TypeDesc;

/// Declares a port for a node kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDecl {
    /// Stable schema key for this port.
    pub key: PortKey,
    /// Direction.
    pub dir: PortDirection,
    /// Kind.
    pub kind: PortKind,
    /// Capacity.
    pub capacity: PortCapacity,
    /// Optional type descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<TypeDesc>,
    /// UI-facing label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Schema for a node kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSchema {
    /// Canonical kind key.
    pub kind: NodeKindKey,
    /// Latest schema version for this kind.
    pub latest_kind_version: u32,
    /// Kind aliases (renames).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kind_aliases: Vec<NodeKindKey>,

    /// UI-facing title.
    pub title: String,
    /// Category path (for create-node search/palette).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category: Vec<String>,
    /// Search keywords.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// Declared ports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortDecl>,

    /// Default node payload.
    #[serde(default)]
    pub default_data: Value,
}

/// Registry for node kinds.
#[derive(Debug, Default, Clone)]
pub struct NodeRegistry {
    by_kind: BTreeMap<NodeKindKey, NodeSchema>,
    by_alias: BTreeMap<NodeKindKey, NodeKindKey>,
}

impl NodeRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a schema.
    ///
    /// Aliases are mapped to the schema's canonical kind.
    pub fn register(&mut self, schema: NodeSchema) {
        for alias in &schema.kind_aliases {
            self.by_alias.insert(alias.clone(), schema.kind.clone());
        }
        self.by_kind.insert(schema.kind.clone(), schema);
    }

    /// Resolves an input kind to a canonical kind (via aliases).
    pub fn resolve_kind<'a>(&'a self, kind: &'a NodeKindKey) -> &'a NodeKindKey {
        self.by_alias.get(kind).unwrap_or(kind)
    }

    /// Looks up a schema by canonical kind key.
    pub fn get(&self, kind: &NodeKindKey) -> Option<&NodeSchema> {
        self.by_kind.get(kind)
    }

    /// Iterates all registered schemas in deterministic order (by kind key).
    pub fn schemas(&self) -> impl Iterator<Item = &NodeSchema> {
        self.by_kind.values()
    }
}
