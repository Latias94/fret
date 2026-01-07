//! Simple built-in profiles.

use std::collections::BTreeSet;

use uuid::Uuid;

use crate::core::{Graph, PortId};
use crate::core::{NodeId, Port, PortCapacity, PortDirection, PortKey, PortKind};
use crate::ops::GraphOpBuilderExt;
use crate::rules::{ConnectPlan, Diagnostic, DiagnosticSeverity, plan_connect_typed};
use crate::types::{DefaultTypeCompatibility, TypeCompatibility, TypeDesc};

use super::GraphProfile;

const VARIADIC_MERGE_KIND: &str = "fret.variadic_merge";
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

    fn concretize(&mut self, graph: &Graph) -> Vec<crate::ops::GraphOp> {
        let _ = self;
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
        .any(|e| e.kind == crate::core::EdgeKind::Data && e.to == port)
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
                ty: None,
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
                ty: None,
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
                    ty: None,
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
    let mut managed: BTreeSet<PortId> = BTreeSet::new();
    for (_, id) in &inputs {
        managed.insert(*id);
    }
    if let Some(out) = output {
        managed.insert(out);
    }

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
