//! Apply helpers for XyFlow-style changes (controlled mode).
//!
//! In ReactFlow, consumers that keep their own node/edge state receive `NodeChange`/`EdgeChange`
//! and apply them via helpers (`applyNodeChanges`, `applyEdgeChanges`).
//!
//! In `fret-node`, the authoritative representation is a `Graph` (hash maps) and undo/redo is
//! powered by reversible `GraphTransaction`. This module provides best-effort, order-preserving
//! application of change events to a `Graph` for controlled integrations.

use std::collections::HashSet;

use crate::core::{Graph, PortId};
use crate::runtime::changes::{EdgeChange, NodeChange, NodeGraphChanges};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ApplyChangesReport {
    pub applied: usize,
    pub ignored: usize,
}

impl ApplyChangesReport {
    pub fn did_change(&self) -> bool {
        self.applied > 0
    }
}

pub fn apply_graph_changes(graph: &mut Graph, changes: &NodeGraphChanges) -> ApplyChangesReport {
    let mut report = apply_node_changes(graph, &changes.nodes);
    let edge_report = apply_edge_changes(graph, &changes.edges);
    report.applied += edge_report.applied;
    report.ignored += edge_report.ignored;
    report
}

pub fn apply_node_changes(graph: &mut Graph, changes: &[NodeChange]) -> ApplyChangesReport {
    let mut report = ApplyChangesReport::default();

    for change in changes {
        match change {
            NodeChange::Add { id, node } => {
                graph.nodes.insert(*id, node.clone());
                report.applied += 1;
            }
            NodeChange::Remove { id } => {
                let Some(removed) = graph.nodes.remove(id) else {
                    report.ignored += 1;
                    continue;
                };

                let port_ids: HashSet<PortId> =
                    removed.ports.iter().copied().collect::<HashSet<_>>();
                if !port_ids.is_empty() {
                    graph.ports.retain(|pid, _| !port_ids.contains(pid));
                    graph
                        .edges
                        .retain(|_, e| !port_ids.contains(&e.from) && !port_ids.contains(&e.to));
                }
                report.applied += 1;
            }
            NodeChange::Position { id, position } => {
                let Some(node) = graph.nodes.get_mut(id) else {
                    report.ignored += 1;
                    continue;
                };
                node.pos = *position;
                report.applied += 1;
            }
            NodeChange::Size { id, size } => {
                let Some(node) = graph.nodes.get_mut(id) else {
                    report.ignored += 1;
                    continue;
                };
                node.size = *size;
                report.applied += 1;
            }
            NodeChange::Collapsed { id, collapsed } => {
                let Some(node) = graph.nodes.get_mut(id) else {
                    report.ignored += 1;
                    continue;
                };
                node.collapsed = *collapsed;
                report.applied += 1;
            }
            NodeChange::Data { id, data } => {
                let Some(node) = graph.nodes.get_mut(id) else {
                    report.ignored += 1;
                    continue;
                };
                node.data = data.clone();
                report.applied += 1;
            }
        }
    }

    report
}

pub fn apply_edge_changes(graph: &mut Graph, changes: &[EdgeChange]) -> ApplyChangesReport {
    let mut report = ApplyChangesReport::default();

    for change in changes {
        match change {
            EdgeChange::Add { id, edge } => {
                graph.edges.insert(*id, edge.clone());
                report.applied += 1;
            }
            EdgeChange::Remove { id } => {
                if graph.edges.remove(id).is_some() {
                    report.applied += 1;
                } else {
                    report.ignored += 1;
                }
            }
            EdgeChange::Kind { id, kind } => {
                let Some(edge) = graph.edges.get_mut(id) else {
                    report.ignored += 1;
                    continue;
                };
                edge.kind = *kind;
                report.applied += 1;
            }
            EdgeChange::Endpoints { id, from, to } => {
                let Some(edge) = graph.edges.get_mut(id) else {
                    report.ignored += 1;
                    continue;
                };
                edge.from = *from;
                edge.to = *to;
                report.applied += 1;
            }
        }
    }

    report
}
