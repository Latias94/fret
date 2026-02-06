use std::collections::BTreeSet;

use serde_json::Value;
use uuid::Uuid;

use super::{Graph, Node, NodeId, SymbolId};

/// Reserved node kind for a "symbol reference node" (blackboard/variable reference).
///
/// This is intentionally a string constant (not an enum) so unknown kinds remain preservable.
pub const SYMBOL_REF_NODE_KIND: &str = "fret.symbol_ref";

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SymbolRefNodeError {
    #[error("symbol ref node missing symbol_id: node={node:?}")]
    MissingSymbolId { node: NodeId },

    #[error("symbol ref node symbol_id is not a string: node={node:?}")]
    SymbolIdNotString { node: NodeId },

    #[error("symbol ref node symbol_id is not a valid uuid: node={node:?} value={value:?}")]
    InvalidSymbolId { node: NodeId, value: String },
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SymbolRefBindingError {
    #[error(
        "symbol ref target is not declared in graph symbols: node={node:?} symbol_id={symbol_id:?}"
    )]
    TargetNotDeclared { node: NodeId, symbol_id: SymbolId },
}

pub fn is_symbol_ref_node(node: &Node) -> bool {
    node.kind.0.as_str() == SYMBOL_REF_NODE_KIND
}

/// Parses the referenced symbol id from a symbol reference node.
///
/// Contract:
/// - `node.kind == SYMBOL_REF_NODE_KIND` implies `node.data` is an object with a `symbol_id` string.
/// - `symbol_id` must parse as a UUID.
pub fn symbol_ref_target_symbol_id(
    node_id: NodeId,
    node: &Node,
) -> Result<Option<SymbolId>, SymbolRefNodeError> {
    if !is_symbol_ref_node(node) {
        return Ok(None);
    }

    let Some(obj) = node.data.as_object() else {
        return Err(SymbolRefNodeError::MissingSymbolId { node: node_id });
    };

    let Some(raw) = obj.get("symbol_id") else {
        return Err(SymbolRefNodeError::MissingSymbolId { node: node_id });
    };

    let Some(s) = raw.as_str() else {
        return Err(SymbolRefNodeError::SymbolIdNotString { node: node_id });
    };

    let uuid = Uuid::parse_str(s).map_err(|_| SymbolRefNodeError::InvalidSymbolId {
        node: node_id,
        value: s.to_string(),
    })?;

    Ok(Some(SymbolId(uuid)))
}

/// Collects all referenced symbol targets in a graph.
///
/// Invalid symbol ref nodes return an error.
pub fn collect_symbol_ref_targets(graph: &Graph) -> Result<BTreeSet<SymbolId>, SymbolRefNodeError> {
    let mut out = BTreeSet::new();
    for (node_id, node) in &graph.nodes {
        if let Some(target) = symbol_ref_target_symbol_id(*node_id, node)? {
            out.insert(target);
        }
    }
    Ok(out)
}

/// Validates that every referenced symbol id is declared in `graph.symbols`.
pub fn validate_symbol_ref_targets_are_declared(
    graph: &Graph,
) -> Result<(), Vec<SymbolRefBindingError>> {
    let mut errors = Vec::new();
    for (node_id, node) in &graph.nodes {
        let Ok(Some(target)) = symbol_ref_target_symbol_id(*node_id, node) else {
            continue;
        };
        if !graph.symbols.contains_key(&target) {
            errors.push(SymbolRefBindingError::TargetNotDeclared {
                node: *node_id,
                symbol_id: target,
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Helper for creating a symbol ref node payload.
pub fn symbol_ref_node_data(symbol_id: SymbolId) -> Value {
    serde_json::json!({ "symbol_id": symbol_id })
}
