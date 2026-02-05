use crate::core::{EdgeId, Graph, GraphId, PortId, SymbolId};
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction, normalize_transaction};

/// Computes a deterministic patch transaction that transforms `from` into `to`.
///
/// This is intended as a collaboration-friendly patch unit and as a conformance gate for refactors.
/// It prefers correctness + determinism over minimality.
pub fn graph_diff(from: &Graph, to: &Graph) -> GraphTransaction {
    let mut tx = GraphTransaction::new();

    diff_imports(from, to, &mut tx);
    diff_symbols(from, to, &mut tx);

    // Nodes/ports/edges: MVP focuses on headless collaboration patching. We keep the phase order
    // apply-safe (edges last because they reference ports).
    diff_nodes(from, to, &mut tx);
    diff_ports(from, to, &mut tx);
    diff_edges(from, to, &mut tx);

    normalize_transaction(tx)
}

fn diff_imports(from: &Graph, to: &Graph, tx: &mut GraphTransaction) {
    for (id, import_to) in &to.imports {
        if let Some(import_from) = from.imports.get(id) {
            if import_from.alias != import_to.alias {
                tx.ops.push(GraphOp::SetImportAlias {
                    id: *id,
                    from: import_from.alias.clone(),
                    to: import_to.alias.clone(),
                });
            }
        } else {
            tx.ops.push(GraphOp::AddImport {
                id: *id,
                import: import_to.clone(),
            });
        }
    }

    for (id, import_from) in &from.imports {
        if !to.imports.contains_key(id) {
            tx.ops.push(GraphOp::RemoveImport {
                id: *id,
                import: import_from.clone(),
            });
        }
    }
}

fn diff_symbols(from: &Graph, to: &Graph, tx: &mut GraphTransaction) {
    for (id, sym_to) in &to.symbols {
        if let Some(sym_from) = from.symbols.get(id) {
            if sym_from.name != sym_to.name {
                tx.ops.push(GraphOp::SetSymbolName {
                    id: *id,
                    from: sym_from.name.clone(),
                    to: sym_to.name.clone(),
                });
            }
            if sym_from.ty != sym_to.ty {
                tx.ops.push(GraphOp::SetSymbolType {
                    id: *id,
                    from: sym_from.ty.clone(),
                    to: sym_to.ty.clone(),
                });
            }
            if sym_from.default_value != sym_to.default_value {
                tx.ops.push(GraphOp::SetSymbolDefaultValue {
                    id: *id,
                    from: sym_from.default_value.clone(),
                    to: sym_to.default_value.clone(),
                });
            }
            if sym_from.meta != sym_to.meta {
                tx.ops.push(GraphOp::SetSymbolMeta {
                    id: *id,
                    from: sym_from.meta.clone(),
                    to: sym_to.meta.clone(),
                });
            }
        } else {
            tx.ops.push(GraphOp::AddSymbol {
                id: *id,
                symbol: sym_to.clone(),
            });
        }
    }

    for (id, sym_from) in &from.symbols {
        if !to.symbols.contains_key(id) {
            tx.ops.push(GraphOp::RemoveSymbol {
                id: *id,
                symbol: sym_from.clone(),
            });
        }
    }
}

fn diff_nodes(from: &Graph, to: &Graph, tx: &mut GraphTransaction) {
    for (id, node_to) in &to.nodes {
        if let Some(node_from) = from.nodes.get(id) {
            if node_from.kind != node_to.kind {
                tx.ops.push(GraphOp::SetNodeKind {
                    id: *id,
                    from: node_from.kind.clone(),
                    to: node_to.kind.clone(),
                });
            }
            if node_from.kind_version != node_to.kind_version {
                tx.ops.push(GraphOp::SetNodeKindVersion {
                    id: *id,
                    from: node_from.kind_version,
                    to: node_to.kind_version,
                });
            }
            if node_from.pos != node_to.pos {
                tx.ops.push(GraphOp::SetNodePos {
                    id: *id,
                    from: node_from.pos,
                    to: node_to.pos,
                });
            }
            if node_from.parent != node_to.parent {
                tx.ops.push(GraphOp::SetNodeParent {
                    id: *id,
                    from: node_from.parent,
                    to: node_to.parent,
                });
            }
            if node_from.size != node_to.size {
                tx.ops.push(GraphOp::SetNodeSize {
                    id: *id,
                    from: node_from.size,
                    to: node_to.size,
                });
            }
            if node_from.collapsed != node_to.collapsed {
                tx.ops.push(GraphOp::SetNodeCollapsed {
                    id: *id,
                    from: node_from.collapsed,
                    to: node_to.collapsed,
                });
            }
            if node_from.ports != node_to.ports {
                tx.ops.push(GraphOp::SetNodePorts {
                    id: *id,
                    from: node_from.ports.clone(),
                    to: node_to.ports.clone(),
                });
            }
            if node_from.data != node_to.data {
                tx.ops.push(GraphOp::SetNodeData {
                    id: *id,
                    from: node_from.data.clone(),
                    to: node_to.data.clone(),
                });
            }
        } else {
            tx.ops.push(GraphOp::AddNode {
                id: *id,
                node: node_to.clone(),
            });
        }
    }

    for (id, node_from) in &from.nodes {
        if !to.nodes.contains_key(id) {
            // Prefer the reversible removal op with captured ports/edges.
            if let Some(op) = crate::ops::GraphOpBuilderExt::build_remove_node_op(from, *id) {
                tx.ops.push(op);
            } else {
                // Fallback: remove node only (should not happen if graph is consistent).
                tx.ops.push(GraphOp::RemoveNode {
                    id: *id,
                    node: node_from.clone(),
                    ports: Vec::new(),
                    edges: Vec::new(),
                });
            }
        }
    }
}

fn diff_ports(from: &Graph, to: &Graph, tx: &mut GraphTransaction) {
    for (id, port_to) in &to.ports {
        if let Some(port_from) = from.ports.get(id) {
            // For MVP we do not have per-port setters yet; fall back to remove+add if different.
            if serde_json::to_value(port_from).ok() != serde_json::to_value(port_to).ok() {
                if let Some(op) = crate::ops::GraphOpBuilderExt::build_remove_port_op(from, *id) {
                    tx.ops.push(op);
                }
                tx.ops.push(GraphOp::AddPort {
                    id: *id,
                    port: port_to.clone(),
                });
            }
        } else {
            tx.ops.push(GraphOp::AddPort {
                id: *id,
                port: port_to.clone(),
            });
        }
    }

    for (id, _port_from) in &from.ports {
        if !to.ports.contains_key(id) {
            if let Some(op) = crate::ops::GraphOpBuilderExt::build_remove_port_op(from, *id) {
                tx.ops.push(op);
            }
        }
    }
}

fn diff_edges(from: &Graph, to: &Graph, tx: &mut GraphTransaction) {
    for (id, edge_to) in &to.edges {
        if let Some(edge_from) = from.edges.get(id) {
            if edge_from.kind != edge_to.kind {
                tx.ops.push(GraphOp::SetEdgeKind {
                    id: *id,
                    from: edge_from.kind,
                    to: edge_to.kind,
                });
            }
            let from_ep = EdgeEndpoints {
                from: edge_from.from,
                to: edge_from.to,
            };
            let to_ep = EdgeEndpoints {
                from: edge_to.from,
                to: edge_to.to,
            };
            if from_ep != to_ep {
                tx.ops.push(GraphOp::SetEdgeEndpoints {
                    id: *id,
                    from: from_ep,
                    to: to_ep,
                });
            }
        } else {
            tx.ops.push(GraphOp::AddEdge {
                id: *id,
                edge: edge_to.clone(),
            });
        }
    }

    for (id, edge_from) in &from.edges {
        if !to.edges.contains_key(id) {
            tx.ops.push(GraphOp::RemoveEdge {
                id: *id,
                edge: edge_from.clone(),
            });
        }
    }
}

// Silence unused warnings for ids we may need in future phases without changing public API.
#[allow(dead_code)]
fn _ids_silence(_a: GraphId, _b: SymbolId, _c: PortId, _d: EdgeId) {}
