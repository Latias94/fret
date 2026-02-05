use crate::core::{Edge, EdgeId, Graph, GraphId, GroupId, NodeId, PortId, StickyNoteId, SymbolId};
use crate::ops::{GraphOp, GraphTransaction};

#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    #[error("node already exists: {id:?}")]
    NodeAlreadyExists { id: NodeId },
    #[error("missing node: {id:?}")]
    MissingNode { id: NodeId },
    #[error("port already exists: {id:?}")]
    PortAlreadyExists { id: PortId },
    #[error("missing port: {id:?}")]
    MissingPort { id: PortId },
    #[error("edge already exists: {id:?}")]
    EdgeAlreadyExists { id: EdgeId },
    #[error("missing edge: {id:?}")]
    MissingEdge { id: EdgeId },
    #[error("symbol already exists: {id:?}")]
    SymbolAlreadyExists { id: SymbolId },
    #[error("missing symbol: {id:?}")]
    MissingSymbol { id: SymbolId },
    #[error("group already exists: {id:?}")]
    GroupAlreadyExists { id: GroupId },
    #[error("missing group: {id:?}")]
    MissingGroup { id: GroupId },
    #[error("node parent references missing group: node={node:?} group={group:?}")]
    NodeParentMissingGroup { node: NodeId, group: GroupId },
    #[error("sticky note already exists: {id:?}")]
    StickyNoteAlreadyExists { id: StickyNoteId },
    #[error("missing sticky note: {id:?}")]
    MissingStickyNote { id: StickyNoteId },
    #[error("node ports list contains unknown port: node={node:?} port={port:?}")]
    NodePortsUnknownPort { node: NodeId, port: PortId },
    #[error("edge references missing port: edge={edge:?} port={port:?}")]
    EdgeMissingPort { edge: EdgeId, port: PortId },
    #[error("import already exists: {id}")]
    ImportAlreadyExists { id: GraphId },
    #[error("missing import: {id}")]
    MissingImport { id: GraphId },
    #[error("remove node op did not match current node: {id:?}")]
    RemoveNodeMismatch { id: NodeId },
    #[error("remove port op did not match current port: {id:?}")]
    RemovePortMismatch { id: PortId },
    #[error("remove edge op did not match current edge: {id:?}")]
    RemoveEdgeMismatch { id: EdgeId },
    #[error("remove symbol op did not match current symbol: {id:?}")]
    RemoveSymbolMismatch { id: SymbolId },
    #[error("remove group op did not match current group: {id:?}")]
    RemoveGroupMismatch { id: GroupId },
    #[error(
        "remove group op expected node parent mismatch: group={group:?} node={node:?} expected={expected:?}"
    )]
    RemoveGroupDetachedMismatch {
        group: GroupId,
        node: NodeId,
        expected: Option<GroupId>,
    },
    #[error("remove sticky note op did not match current note: {id:?}")]
    RemoveStickyNoteMismatch { id: StickyNoteId },
}

pub fn apply_transaction(graph: &mut Graph, tx: &GraphTransaction) -> Result<(), ApplyError> {
    for op in &tx.ops {
        apply_op(graph, op)?;
    }
    Ok(())
}

pub fn apply_op(graph: &mut Graph, op: &GraphOp) -> Result<(), ApplyError> {
    match op {
        GraphOp::AddNode { id, node } => {
            if graph.nodes.contains_key(id) {
                return Err(ApplyError::NodeAlreadyExists { id: *id });
            }
            graph.nodes.insert(*id, node.clone());
        }
        GraphOp::RemoveNode {
            id,
            node,
            ports,
            edges,
        } => {
            let Some(current) = graph.nodes.get(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            if current.kind != node.kind || current.kind_version != node.kind_version {
                return Err(ApplyError::RemoveNodeMismatch { id: *id });
            }

            for (edge_id, edge) in edges {
                remove_edge_exact(graph, *edge_id, edge)?;
            }
            for (port_id, port) in ports {
                remove_port_exact(graph, *port_id, port)?;
            }

            graph.nodes.remove(id);
        }
        GraphOp::SetNodePos { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            node.pos = *to;
        }
        GraphOp::SetNodeKind { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            node.kind = to.clone();
        }
        GraphOp::SetNodeKindVersion { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            node.kind_version = *to;
        }
        GraphOp::SetNodeParent { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            if let Some(group) = to
                && !graph.groups.contains_key(group)
            {
                return Err(ApplyError::NodeParentMissingGroup {
                    node: *id,
                    group: *group,
                });
            }
            node.parent = *to;
        }
        GraphOp::SetNodeSize { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            node.size = *to;
        }
        GraphOp::SetNodeCollapsed { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            node.collapsed = *to;
        }
        GraphOp::SetNodePorts { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            for port_id in to {
                let Some(port) = graph.ports.get(port_id) else {
                    return Err(ApplyError::NodePortsUnknownPort {
                        node: *id,
                        port: *port_id,
                    });
                };
                if port.node != *id {
                    return Err(ApplyError::NodePortsUnknownPort {
                        node: *id,
                        port: *port_id,
                    });
                }
            }
            node.ports = to.clone();
        }
        GraphOp::SetNodeData { id, to, .. } => {
            let Some(node) = graph.nodes.get_mut(id) else {
                return Err(ApplyError::MissingNode { id: *id });
            };
            node.data = to.clone();
        }
        GraphOp::AddPort { id, port } => {
            if graph.ports.contains_key(id) {
                return Err(ApplyError::PortAlreadyExists { id: *id });
            }
            if !graph.nodes.contains_key(&port.node) {
                return Err(ApplyError::MissingNode { id: port.node });
            }
            graph.ports.insert(*id, port.clone());
        }
        GraphOp::RemovePort { id, port, edges } => {
            let Some(current) = graph.ports.get(id) else {
                return Err(ApplyError::MissingPort { id: *id });
            };
            if current.node != port.node || current.key != port.key {
                return Err(ApplyError::RemovePortMismatch { id: *id });
            }
            for (edge_id, edge) in edges {
                remove_edge_exact(graph, *edge_id, edge)?;
            }
            graph.ports.remove(id);
            if let Some(node) = graph.nodes.get_mut(&port.node) {
                node.ports.retain(|p| p != id);
            }
        }
        GraphOp::SetPortConnectable { id, to, .. } => {
            let Some(port) = graph.ports.get_mut(id) else {
                return Err(ApplyError::MissingPort { id: *id });
            };
            port.connectable = *to;
        }
        GraphOp::SetPortConnectableStart { id, to, .. } => {
            let Some(port) = graph.ports.get_mut(id) else {
                return Err(ApplyError::MissingPort { id: *id });
            };
            port.connectable_start = *to;
        }
        GraphOp::SetPortConnectableEnd { id, to, .. } => {
            let Some(port) = graph.ports.get_mut(id) else {
                return Err(ApplyError::MissingPort { id: *id });
            };
            port.connectable_end = *to;
        }
        GraphOp::SetPortType { id, to, .. } => {
            let Some(port) = graph.ports.get_mut(id) else {
                return Err(ApplyError::MissingPort { id: *id });
            };
            port.ty = to.clone();
        }
        GraphOp::SetPortData { id, to, .. } => {
            let Some(port) = graph.ports.get_mut(id) else {
                return Err(ApplyError::MissingPort { id: *id });
            };
            port.data = to.clone();
        }
        GraphOp::AddEdge { id, edge } => {
            if graph.edges.contains_key(id) {
                return Err(ApplyError::EdgeAlreadyExists { id: *id });
            }
            if !graph.ports.contains_key(&edge.from) {
                return Err(ApplyError::EdgeMissingPort {
                    edge: *id,
                    port: edge.from,
                });
            }
            if !graph.ports.contains_key(&edge.to) {
                return Err(ApplyError::EdgeMissingPort {
                    edge: *id,
                    port: edge.to,
                });
            }
            graph.edges.insert(*id, edge.clone());
        }
        GraphOp::RemoveEdge { id, edge } => {
            remove_edge_exact(graph, *id, edge)?;
        }
        GraphOp::SetEdgeKind { id, to, .. } => {
            let Some(edge) = graph.edges.get_mut(id) else {
                return Err(ApplyError::MissingEdge { id: *id });
            };
            edge.kind = *to;
        }
        GraphOp::SetEdgeSelectable { id, to, .. } => {
            let Some(edge) = graph.edges.get_mut(id) else {
                return Err(ApplyError::MissingEdge { id: *id });
            };
            edge.selectable = *to;
        }
        GraphOp::SetEdgeDeletable { id, to, .. } => {
            let Some(edge) = graph.edges.get_mut(id) else {
                return Err(ApplyError::MissingEdge { id: *id });
            };
            edge.deletable = *to;
        }
        GraphOp::SetEdgeReconnectable { id, to, .. } => {
            let Some(edge) = graph.edges.get_mut(id) else {
                return Err(ApplyError::MissingEdge { id: *id });
            };
            edge.reconnectable = *to;
        }
        GraphOp::SetEdgeEndpoints { id, to, .. } => {
            let Some(edge) = graph.edges.get_mut(id) else {
                return Err(ApplyError::MissingEdge { id: *id });
            };
            if !graph.ports.contains_key(&to.from) {
                return Err(ApplyError::EdgeMissingPort {
                    edge: *id,
                    port: to.from,
                });
            }
            if !graph.ports.contains_key(&to.to) {
                return Err(ApplyError::EdgeMissingPort {
                    edge: *id,
                    port: to.to,
                });
            }
            edge.from = to.from;
            edge.to = to.to;
        }
        GraphOp::AddImport { id, import } => {
            if graph.imports.contains_key(id) {
                return Err(ApplyError::ImportAlreadyExists { id: *id });
            }
            graph.imports.insert(*id, import.clone());
        }
        GraphOp::RemoveImport { id, .. } => {
            if !graph.imports.contains_key(id) {
                return Err(ApplyError::MissingImport { id: *id });
            }
            graph.imports.remove(id);
        }
        GraphOp::SetImportAlias { id, to, .. } => {
            let Some(import) = graph.imports.get_mut(id) else {
                return Err(ApplyError::MissingImport { id: *id });
            };
            import.alias = to.clone();
        }
        GraphOp::AddSymbol { id, symbol } => {
            if graph.symbols.contains_key(id) {
                return Err(ApplyError::SymbolAlreadyExists { id: *id });
            }
            graph.symbols.insert(*id, symbol.clone());
        }
        GraphOp::RemoveSymbol { id, symbol } => {
            let Some(current) = graph.symbols.get(id) else {
                return Err(ApplyError::MissingSymbol { id: *id });
            };
            if current.name != symbol.name {
                return Err(ApplyError::RemoveSymbolMismatch { id: *id });
            }
            graph.symbols.remove(id);
        }
        GraphOp::SetSymbolName { id, to, .. } => {
            let Some(symbol) = graph.symbols.get_mut(id) else {
                return Err(ApplyError::MissingSymbol { id: *id });
            };
            symbol.name = to.clone();
        }
        GraphOp::SetSymbolType { id, to, .. } => {
            let Some(symbol) = graph.symbols.get_mut(id) else {
                return Err(ApplyError::MissingSymbol { id: *id });
            };
            symbol.ty = to.clone();
        }
        GraphOp::SetSymbolDefaultValue { id, to, .. } => {
            let Some(symbol) = graph.symbols.get_mut(id) else {
                return Err(ApplyError::MissingSymbol { id: *id });
            };
            symbol.default_value = to.clone();
        }
        GraphOp::SetSymbolMeta { id, to, .. } => {
            let Some(symbol) = graph.symbols.get_mut(id) else {
                return Err(ApplyError::MissingSymbol { id: *id });
            };
            symbol.meta = to.clone();
        }
        GraphOp::AddGroup { id, group } => {
            if graph.groups.contains_key(id) {
                return Err(ApplyError::GroupAlreadyExists { id: *id });
            }
            graph.groups.insert(*id, group.clone());
        }
        GraphOp::RemoveGroup {
            id,
            group,
            detached,
        } => {
            let Some(current) = graph.groups.get(id) else {
                return Err(ApplyError::MissingGroup { id: *id });
            };
            if current.title != group.title {
                return Err(ApplyError::RemoveGroupMismatch { id: *id });
            }

            for (node_id, expected_parent) in detached {
                let Some(node) = graph.nodes.get_mut(node_id) else {
                    return Err(ApplyError::MissingNode { id: *node_id });
                };
                if node.parent != *expected_parent {
                    return Err(ApplyError::RemoveGroupDetachedMismatch {
                        group: *id,
                        node: *node_id,
                        expected: *expected_parent,
                    });
                }
                node.parent = None;
            }

            graph.groups.remove(id);
        }
        GraphOp::SetGroupRect { id, to, .. } => {
            let Some(group) = graph.groups.get_mut(id) else {
                return Err(ApplyError::MissingGroup { id: *id });
            };
            group.rect = *to;
        }
        GraphOp::SetGroupTitle { id, to, .. } => {
            let Some(group) = graph.groups.get_mut(id) else {
                return Err(ApplyError::MissingGroup { id: *id });
            };
            group.title = to.clone();
        }
        GraphOp::AddStickyNote { id, note } => {
            if graph.sticky_notes.contains_key(id) {
                return Err(ApplyError::StickyNoteAlreadyExists { id: *id });
            }
            graph.sticky_notes.insert(*id, note.clone());
        }
        GraphOp::RemoveStickyNote { id, note } => {
            let Some(current) = graph.sticky_notes.get(id) else {
                return Err(ApplyError::MissingStickyNote { id: *id });
            };
            if current.text != note.text {
                return Err(ApplyError::RemoveStickyNoteMismatch { id: *id });
            }
            graph.sticky_notes.remove(id);
        }
    }
    Ok(())
}

fn remove_edge_exact(graph: &mut Graph, id: EdgeId, expected: &Edge) -> Result<(), ApplyError> {
    let Some(current) = graph.edges.get(&id) else {
        return Err(ApplyError::MissingEdge { id });
    };
    if current.kind != expected.kind || current.from != expected.from || current.to != expected.to {
        return Err(ApplyError::RemoveEdgeMismatch { id });
    }
    graph.edges.remove(&id);
    Ok(())
}

fn remove_port_exact(
    graph: &mut Graph,
    id: PortId,
    expected: &crate::core::Port,
) -> Result<(), ApplyError> {
    let Some(current) = graph.ports.get(&id) else {
        return Err(ApplyError::MissingPort { id });
    };
    if current.node != expected.node || current.key != expected.key {
        return Err(ApplyError::RemovePortMismatch { id });
    }
    graph.ports.remove(&id);
    Ok(())
}
