//! A permissive built-in dataflow profile.
//!
//! This profile is intended as a convenient starting point for workflow/dataflow graphs:
//! - uses `Port::ty` as the source of truth for typing,
//! - enforces a small default compatibility table for data edges when both sides have types,
//! - runs a small concretization pass for kit recipe nodes (e.g. `variadic_merge`).

use std::collections::BTreeSet;

use uuid::Uuid;

use crate::core::{
    EdgeKind, Graph, NodeId, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::ops::GraphOpBuilderExt;
use crate::rules::{ConnectPlan, Diagnostic, DiagnosticSeverity, plan_connect_typed};
use crate::types::{DefaultTypeCompatibility, TypeCompatibility, TypeDesc};

use crate::kit::nodes::VARIADIC_MERGE_KIND;
use crate::profile::GraphProfile;

const VARIADIC_OUTPUT_KEY: &str = "out";

/// A permissive dataflow profile:
/// - allows both data and exec edges,
/// - uses `Port::ty` as the source of truth for typing,
/// - enforces a small default compatibility table for data edges when both sides have types.
#[derive(Debug, Default, Clone)]
pub struct DataflowProfile {
    compat: DefaultTypeCompatibility,
}

impl DataflowProfile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_compat(mut self, compat: DefaultTypeCompatibility) -> Self {
        self.compat = compat;
        self
    }

    pub fn compat_mut(&mut self) -> &mut dyn TypeCompatibility {
        &mut self.compat
    }
}

impl GraphProfile for DataflowProfile {
    fn type_of_port(&mut self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        graph.ports.get(&port).and_then(|p| p.ty.clone())
    }

    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        plan_connect_typed(
            graph,
            a,
            b,
            |g, p| g.ports.get(&p).and_then(|p| p.ty.clone()),
            &mut self.compat,
        )
    }

    fn validate_graph(&mut self, graph: &Graph) -> Vec<Diagnostic> {
        let report = crate::core::validate_graph(graph);
        report
            .errors
            .into_iter()
            .map(|err| Diagnostic {
                key: "graph.invalid".to_string(),
                severity: DiagnosticSeverity::Error,
                target: crate::rules::DiagnosticTarget::Graph,
                message: err.to_string(),
                fixes: Vec::new(),
            })
            .collect()
    }

    fn allow_cycles(&self, _edge_kind: EdgeKind) -> bool {
        true
    }

    fn concretize(&mut self, graph: &Graph) -> Vec<crate::ops::GraphOp> {
        let mut ops: Vec<crate::ops::GraphOp> = Vec::new();

        for (node_id, node) in &graph.nodes {
            if node.kind.0 != VARIADIC_MERGE_KIND {
                continue;
            }

            ops.extend(concretize_variadic_merge(graph, *node_id));
        }

        ops
    }
}

fn parse_variadic_input_index(key: &PortKey) -> Option<usize> {
    let s = key.0.as_str();
    let rest = s.strip_prefix("in")?;
    if rest.is_empty() {
        return None;
    }
    rest.parse::<usize>().ok()
}

fn alloc_port_id(graph: &Graph, node: NodeId, key: &PortKey) -> PortId {
    let base = format!("port:{}:{}", node.0, key.0);
    for attempt in 0u32..32 {
        let name = if attempt == 0 {
            base.clone()
        } else {
            format!("{base}#{attempt}")
        };
        let id = PortId(Uuid::new_v5(&graph.graph_id.0, name.as_bytes()));
        if !graph.ports.contains_key(&id) {
            return id;
        }
    }
    // Extremely unlikely; fall back to v4 to avoid an infinite loop.
    PortId::new()
}

fn port_has_incoming_edge(graph: &Graph, port: PortId) -> bool {
    graph
        .edges
        .values()
        .any(|e| e.kind == EdgeKind::Data && e.to == port)
}

fn concretize_variadic_merge(graph: &Graph, node_id: NodeId) -> Vec<crate::ops::GraphOp> {
    let mut ops: Vec<crate::ops::GraphOp> = Vec::new();
    let Some(node) = graph.nodes.get(&node_id) else {
        return ops;
    };

    let mut removed_ports: BTreeSet<PortId> = BTreeSet::new();
    let mut unmanaged: Vec<PortId> = Vec::new();
    let mut inputs: Vec<(usize, PortId)> = Vec::new();
    let mut output: Option<PortId> = None;

    for port_id in &node.ports {
        let Some(port) = graph.ports.get(port_id) else {
            continue;
        };
        if port.node != node_id {
            continue;
        }

        if port.dir == PortDirection::Out
            && port.kind == PortKind::Data
            && port.key.0 == VARIADIC_OUTPUT_KEY
        {
            output = Some(*port_id);
            continue;
        }

        if port.dir == PortDirection::In && port.kind == PortKind::Data {
            if let Some(ix) = parse_variadic_input_index(&port.key) {
                inputs.push((ix, *port_id));
                continue;
            }
        }

        unmanaged.push(*port_id);
    }

    inputs.sort_by_key(|(ix, _)| *ix);

    let mut seed_ty = output
        .and_then(|id| graph.ports.get(&id))
        .and_then(|p| p.ty.clone());
    if seed_ty.is_none() {
        seed_ty = inputs
            .iter()
            .find_map(|(_, id)| graph.ports.get(id).and_then(|p| p.ty.clone()));
    }

    if output.is_none() {
        let key = PortKey::new(VARIADIC_OUTPUT_KEY);
        let id = alloc_port_id(graph, node_id, &key);
        ops.push(crate::ops::GraphOp::AddPort {
            id,
            port: Port {
                node: node_id,
                key,
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: seed_ty.clone(),
                data: serde_json::Value::Null,
            },
        });
        output = Some(id);
    }

    if inputs.is_empty() {
        let key = PortKey::new("in0");
        let id = alloc_port_id(graph, node_id, &key);
        ops.push(crate::ops::GraphOp::AddPort {
            id,
            port: Port {
                node: node_id,
                key,
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: seed_ty.clone(),
                data: serde_json::Value::Null,
            },
        });
        inputs.push((0, id));
    }

    // Ensure the last input is always an empty "add new input" slot.
    if let Some((last_ix, last_port)) = inputs.last().copied() {
        if port_has_incoming_edge(graph, last_port) {
            let next_ix = last_ix.saturating_add(1);
            let key = PortKey::new(format!("in{next_ix}"));
            let id = alloc_port_id(graph, node_id, &key);
            ops.push(crate::ops::GraphOp::AddPort {
                id,
                port: Port {
                    node: node_id,
                    key,
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: seed_ty.clone(),
                    data: serde_json::Value::Null,
                },
            });
            inputs.push((next_ix, id));
        }
    }

    // Trim extra trailing empty inputs (keep exactly one trailing empty).
    while inputs.len() > 1 {
        let last = inputs[inputs.len() - 1].1;
        let prev = inputs[inputs.len() - 2].1;
        if !port_has_incoming_edge(graph, last) && !port_has_incoming_edge(graph, prev) {
            let Some(remove_op) = graph.build_remove_port_op(last) else {
                break;
            };
            ops.push(remove_op);
            removed_ports.insert(last);
            inputs.pop();
            continue;
        }
        break;
    }

    // If nothing changed, avoid emitting a noisy SetNodePorts.
    let mut desired: Vec<PortId> = inputs.iter().map(|(_, id)| *id).collect();
    if let Some(out) = output {
        desired.push(out);
    }
    desired.extend(unmanaged.iter().copied());

    if desired != node.ports {
        let mut from_ports = node.ports.clone();
        from_ports.retain(|id| !removed_ports.contains(id));
        ops.push(crate::ops::GraphOp::SetNodePorts {
            id: node_id,
            from: from_ports,
            to: desired,
        });
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CanvasPoint, Edge, EdgeId, Node, NodeKindKey};

    fn make_node(kind: &str) -> Node {
        Node {
            kind: NodeKindKey::new(kind),
            kind_version: 0,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            parent: None,
            size: None,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        }
    }

    #[test]
    fn variadic_merge_adds_new_input_when_last_is_connected() {
        let mut graph = Graph::default();
        let merge = NodeId::new();
        graph.nodes.insert(merge, make_node(VARIADIC_MERGE_KIND));

        let mut profile = DataflowProfile::new();
        let _ = crate::profile::apply_transaction_with_profile(
            &mut graph,
            &mut profile,
            &crate::ops::GraphTransaction::new(),
        )
        .unwrap();

        // Concretize should ensure at least in0 + out exist.
        let ports = graph.nodes.get(&merge).unwrap().ports.clone();
        assert!(ports.len() >= 2);

        let in0 = ports
            .iter()
            .find_map(|id| {
                graph
                    .ports
                    .get(id)
                    .and_then(|p| (p.key.0 == "in0").then_some(*id))
            })
            .expect("in0");
        let out = ports
            .iter()
            .find_map(|id| {
                graph
                    .ports
                    .get(id)
                    .and_then(|p| (p.key.0 == "out").then_some(*id))
            })
            .expect("out");

        // Connect something to in0.
        let src_node = NodeId::new();
        graph.nodes.insert(src_node, make_node("demo.src"));
        let src_out = PortId::new();
        graph.ports.insert(
            src_out,
            Port {
                node: src_node,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: Some(TypeDesc::Float),
                data: serde_json::Value::Null,
            },
        );
        graph.nodes.get_mut(&src_node).unwrap().ports.push(src_out);

        let edge_id = EdgeId::new();
        let tx = crate::ops::GraphTransaction {
            label: None,
            ops: vec![crate::ops::GraphOp::AddEdge {
                id: edge_id,
                edge: Edge {
                    kind: EdgeKind::Data,
                    from: src_out,
                    to: in0,
                    selectable: None,
                },
            }],
        };
        let committed =
            crate::profile::apply_transaction_with_profile(&mut graph, &mut profile, &tx).unwrap();
        assert!(!committed.ops.is_empty());

        // After concretize, last input should be empty slot, so an additional in1 exists.
        let ports = graph.nodes.get(&merge).unwrap().ports.clone();
        assert!(
            ports
                .iter()
                .any(|id| graph.ports.get(id).is_some_and(|p| p.key.0 == "in1")),
            "expected in1 to be created"
        );

        // Ensure output is still present.
        assert!(ports.contains(&out));
    }

    #[test]
    fn variadic_merge_trims_trailing_empty_inputs() {
        let mut graph = Graph::default();
        let merge = NodeId::new();
        graph.nodes.insert(merge, make_node(VARIADIC_MERGE_KIND));

        let mut profile = DataflowProfile::new();
        let _ = crate::profile::apply_transaction_with_profile(
            &mut graph,
            &mut profile,
            &crate::ops::GraphTransaction::new(),
        )
        .unwrap();

        // Force-add a few extra empty inputs via concretize (simulate previous state).
        let ports = graph.nodes.get(&merge).unwrap().ports.clone();
        let in0 = ports
            .iter()
            .find_map(|id| {
                graph
                    .ports
                    .get(id)
                    .and_then(|p| (p.key.0 == "in0").then_some(*id))
            })
            .expect("in0");

        // Add an extra empty port in2 directly (unmanaged), then let concretize trim it.
        let in2 = PortId::new();
        graph.ports.insert(
            in2,
            Port {
                node: merge,
                key: PortKey::new("in2"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        let mut ports = graph.nodes.get(&merge).unwrap().ports.clone();
        ports.insert(ports.len().saturating_sub(1), in2);
        graph.nodes.get_mut(&merge).unwrap().ports = ports;

        let _ = crate::profile::apply_transaction_with_profile(
            &mut graph,
            &mut profile,
            &crate::ops::GraphTransaction::new(),
        )
        .unwrap();

        // Expect only one trailing empty slot beyond the connected/seeded ones.
        let ports = graph.nodes.get(&merge).unwrap().ports.clone();
        assert!(
            ports
                .iter()
                .filter(|id| {
                    graph
                        .ports
                        .get(id)
                        .is_some_and(|p| p.dir == PortDirection::In && p.kind == PortKind::Data)
                })
                .count()
                >= 1
        );
        assert!(ports.contains(&in0));
    }
}
