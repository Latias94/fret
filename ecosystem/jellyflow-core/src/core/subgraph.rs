use std::collections::BTreeSet;

use serde_json::Value;
use uuid::Uuid;

use super::{Graph, GraphId, Node, NodeId};

/// Reserved node kind for a "subgraph node".
///
/// This is intentionally a string constant (not an enum) so unknown kinds remain preservable.
pub const SUBGRAPH_NODE_KIND: &str = "fret.subgraph";

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SubgraphNodeError {
    #[error("subgraph node missing graph_id: node={node:?}")]
    MissingGraphId { node: NodeId },

    #[error("subgraph node graph_id is not a string: node={node:?}")]
    GraphIdNotString { node: NodeId },

    #[error("subgraph node graph_id is not a valid uuid: node={node:?} value={value:?}")]
    InvalidGraphId { node: NodeId, value: String },
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SubgraphBindingError {
    #[error("subgraph target is not declared in graph imports: node={node:?} graph_id={graph_id}")]
    TargetNotImported { node: NodeId, graph_id: GraphId },
}

pub fn is_subgraph_node(node: &Node) -> bool {
    node.kind.0.as_str() == SUBGRAPH_NODE_KIND
}

/// Parses the referenced graph id from a subgraph node.
///
/// Contract:
/// - `node.kind == SUBGRAPH_NODE_KIND` implies `node.data` is an object with a `graph_id` string.
/// - `graph_id` must parse as a UUID.
pub fn subgraph_target_graph_id(
    node_id: NodeId,
    node: &Node,
) -> Result<Option<GraphId>, SubgraphNodeError> {
    if !is_subgraph_node(node) {
        return Ok(None);
    }

    let Some(obj) = node.data.as_object() else {
        return Err(SubgraphNodeError::MissingGraphId { node: node_id });
    };

    let Some(raw) = obj.get("graph_id") else {
        return Err(SubgraphNodeError::MissingGraphId { node: node_id });
    };

    let Some(s) = raw.as_str() else {
        return Err(SubgraphNodeError::GraphIdNotString { node: node_id });
    };

    let uuid = Uuid::parse_str(s).map_err(|_| SubgraphNodeError::InvalidGraphId {
        node: node_id,
        value: s.to_string(),
    })?;

    Ok(Some(GraphId(uuid)))
}

/// Collects all referenced subgraph targets in a graph.
///
/// Invalid subgraph nodes return an error.
pub fn collect_subgraph_targets(graph: &Graph) -> Result<BTreeSet<GraphId>, SubgraphNodeError> {
    let mut out = BTreeSet::new();
    for (node_id, node) in &graph.nodes {
        if let Some(target) = subgraph_target_graph_id(*node_id, node)? {
            out.insert(target);
        }
    }
    Ok(out)
}

/// Validates that every referenced subgraph target graph id is declared in `graph.imports`.
pub fn validate_subgraph_targets_are_imported(
    graph: &Graph,
) -> Result<(), Vec<SubgraphBindingError>> {
    let mut errors = Vec::new();
    for (node_id, node) in &graph.nodes {
        let Ok(Some(target)) = subgraph_target_graph_id(*node_id, node) else {
            continue;
        };
        if !graph.imports.contains_key(&target) {
            errors.push(SubgraphBindingError::TargetNotImported {
                node: *node_id,
                graph_id: target,
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Helper for creating a subgraph node payload.
pub fn subgraph_node_data(graph_id: GraphId) -> Value {
    serde_json::json!({ "graph_id": graph_id })
}
