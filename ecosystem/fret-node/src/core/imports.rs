use std::collections::{BTreeMap, BTreeSet};

use super::{Graph, GraphId};

/// Reserved metadata for a graph import.
///
/// The presence of an entry declares a dependency from the importing graph onto the imported
/// graph. Additional metadata (aliasing, pinning, namespaces) can be added later without changing
/// the key contract.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GraphImport {
    /// Optional import alias for UI and authoring ergonomics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum GraphImportError {
    #[error("import references missing graph: from={from} to={to}")]
    MissingGraph { from: GraphId, to: GraphId },

    #[error("import cycle detected")]
    Cycle { cycle: Vec<GraphId> },
}

#[derive(Debug, Clone)]
pub struct GraphImportClosure {
    /// All reachable imports (excluding the root graph id).
    pub reachable: BTreeSet<GraphId>,
    /// Deterministic DFS postorder (dependencies first), excluding the root graph id.
    pub order: Vec<GraphId>,
}

impl GraphImportClosure {
    pub fn contains(&self, graph: GraphId) -> bool {
        self.reachable.contains(&graph)
    }
}

/// Resolves the transitive import closure for a graph id with deterministic semantics.
///
/// Determinism contract:
/// - imports are traversed in `BTreeMap` key order (`GraphId` total order),
/// - the resulting `order` is DFS postorder (dependencies appear before dependents).
pub fn resolve_import_closure<'a>(
    root_graph: &'a Graph,
    mut resolver: impl FnMut(GraphId) -> Option<&'a Graph>,
) -> Result<GraphImportClosure, GraphImportError> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Temporary,
        Permanent,
    }

    let root = root_graph.graph_id;
    let mut marks: BTreeMap<GraphId, Mark> = BTreeMap::new();
    let mut stack: Vec<GraphId> = Vec::new();
    let mut closure = GraphImportClosure {
        reachable: BTreeSet::new(),
        order: Vec::new(),
    };

    fn visit<'a>(
        node: GraphId,
        from: GraphId,
        root: GraphId,
        root_graph: &'a Graph,
        resolver: &mut impl FnMut(GraphId) -> Option<&'a Graph>,
        marks: &mut BTreeMap<GraphId, Mark>,
        stack: &mut Vec<GraphId>,
        closure: &mut GraphImportClosure,
    ) -> Result<(), GraphImportError> {
        match marks.get(&node).copied() {
            Some(Mark::Permanent) => return Ok(()),
            Some(Mark::Temporary) => {
                if let Some(pos) = stack.iter().position(|id| *id == node) {
                    let mut cycle = stack[pos..].to_vec();
                    cycle.push(node);
                    return Err(GraphImportError::Cycle { cycle });
                }
                return Err(GraphImportError::Cycle {
                    cycle: vec![node, node],
                });
            }
            None => {}
        }

        marks.insert(node, Mark::Temporary);
        stack.push(node);

        let map: &BTreeMap<GraphId, GraphImport> = if node == root {
            &root_graph.imports
        } else {
            let Some(graph) = resolver(node) else {
                return Err(GraphImportError::MissingGraph { from, to: node });
            };
            &graph.imports
        };

        for (dep, _meta) in map {
            closure.reachable.insert(*dep);
            visit(
                *dep, node, root, root_graph, resolver, marks, stack, closure,
            )?;
        }

        stack.pop();
        marks.insert(node, Mark::Permanent);

        if node != root {
            closure.order.push(node);
        }

        Ok(())
    }

    visit(
        root,
        root,
        root,
        root_graph,
        &mut resolver,
        &mut marks,
        &mut stack,
        &mut closure,
    )?;

    Ok(closure)
}
