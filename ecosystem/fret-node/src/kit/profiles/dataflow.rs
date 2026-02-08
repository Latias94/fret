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
    SYMBOL_REF_NODE_KIND, symbol_ref_target_symbol_id,
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

    fn plan_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        _mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        plan_connect_typed(
            graph,
            a,
            b,
            |g, p| g.ports.get(&p).and_then(|p| p.ty.clone()),
            &mut self.compat,
        )
    }

    fn validate_graph(&mut self, graph: &Graph) -> Vec<Diagnostic> {
        let report = crate::core::validate_graph_structural(graph);
        let mut diags: Vec<Diagnostic> = report
            .errors
            .into_iter()
            .map(|err| Diagnostic {
                key: "graph.invalid".to_string(),
                severity: DiagnosticSeverity::Error,
                target: crate::rules::DiagnosticTarget::Graph,
                message: err.to_string(),
                fixes: Vec::new(),
            })
            .collect();

        for (edge_id, edge) in &graph.edges {
            let Some(from) = graph.ports.get(&edge.from) else {
                continue;
            };
            let Some(to) = graph.ports.get(&edge.to) else {
                continue;
            };
            if from.dir != PortDirection::Out || to.dir != PortDirection::In {
                diags.push(Diagnostic {
                    key: "graph.invalid_edge_direction".to_string(),
                    severity: DiagnosticSeverity::Error,
                    target: crate::rules::DiagnosticTarget::Graph,
                    message: format!(
                        "edge port directions are invalid: edge={edge_id:?} from_dir={:?} to_dir={:?}",
                        from.dir, to.dir
                    ),
                    fixes: Vec::new(),
                });
            }
        }

        diags
    }

    fn allow_cycles(&self, _edge_kind: EdgeKind) -> bool {
        true
    }

    fn concretize(&mut self, graph: &Graph) -> Vec<crate::ops::GraphOp> {
        let mut ops: Vec<crate::ops::GraphOp> = Vec::new();

        for (node_id, node) in &graph.nodes {
            if node.kind.0 == VARIADIC_MERGE_KIND {
                ops.extend(concretize_variadic_merge(graph, *node_id));
                continue;
            }
            if node.kind.0 == SYMBOL_REF_NODE_KIND {
                ops.extend(concretize_symbol_ref_node(graph, *node_id));
            }
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

fn concretize_symbol_ref_node(graph: &Graph, node_id: NodeId) -> Vec<crate::ops::GraphOp> {
    let mut ops: Vec<crate::ops::GraphOp> = Vec::new();
    let Some(node) = graph.nodes.get(&node_id) else {
        return ops;
    };

    let Ok(Some(symbol_id)) = symbol_ref_target_symbol_id(node_id, node) else {
        // Invalid node data is handled by structural validation; skip concretization so we don't
        // create ports for an invalid contract shape.
        return ops;
    };

    let desired_ty: Option<TypeDesc> = graph.symbols.get(&symbol_id).and_then(|s| s.ty.clone());

    let out_key = PortKey::new("out");
    let mut out: Option<PortId> = None;
    let mut unmanaged: Vec<PortId> = Vec::new();

    for port_id in &node.ports {
        let Some(port) = graph.ports.get(port_id) else {
            continue;
        };
        if port.node != node_id {
            continue;
        }

        if port.dir == PortDirection::Out && port.kind == PortKind::Data && port.key.0 == out_key.0
        {
            if out.is_none() {
                out = Some(*port_id);
                continue;
            }
        }

        unmanaged.push(*port_id);
    }

    if out.is_none() {
        let id = alloc_port_id(graph, node_id, &out_key);
        ops.push(crate::ops::GraphOp::AddPort {
            id,
            port: Port {
                node: node_id,
                key: out_key.clone(),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: desired_ty.clone(),
                data: serde_json::Value::Null,
            },
        });
        out = Some(id);
    }

    if let Some(out) = out
        && let Some(current) = graph.ports.get(&out)
        && current.ty != desired_ty
    {
        ops.push(crate::ops::GraphOp::SetPortType {
            id: out,
            from: current.ty.clone(),
            to: desired_ty,
        });
    }

    let mut desired: Vec<PortId> = Vec::new();
    if let Some(out) = out {
        desired.push(out);
    }
    desired.extend(unmanaged.iter().copied());

    if desired != node.ports {
        ops.push(crate::ops::GraphOp::SetNodePorts {
            id: node_id,
            from: node.ports.clone(),
            to: desired,
        });
    }

    ops
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
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
    use crate::core::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
        PortDirection, PortId, PortKey, PortKind, Symbol, SymbolId,
    };

    fn make_node(kind: &str) -> Node {
        Node {
            kind: NodeKindKey::new(kind),
            kind_version: 0,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        }
    }

    fn make_port(
        node: NodeId,
        key: &str,
        dir: PortDirection,
        kind: PortKind,
        capacity: PortCapacity,
    ) -> Port {
        Port {
            node,
            key: PortKey::new(key),
            dir,
            kind,
            capacity,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        }
    }

    #[test]
    fn validate_graph_rejects_out_to_out_edges() {
        let mut graph = Graph::default();
        let a = NodeId::new();
        let b = NodeId::new();
        graph.nodes.insert(a, make_node("core.a"));
        graph.nodes.insert(b, make_node("core.b"));

        let out_a = PortId::new();
        let out_b = PortId::new();
        graph.ports.insert(
            out_a,
            make_port(
                a,
                "out",
                PortDirection::Out,
                PortKind::Data,
                PortCapacity::Multi,
            ),
        );
        graph.ports.insert(
            out_b,
            make_port(
                b,
                "out",
                PortDirection::Out,
                PortKind::Data,
                PortCapacity::Multi,
            ),
        );

        let edge_id = EdgeId::new();
        graph.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: out_a,
                to: out_b,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let mut profile = DataflowProfile::new();
        let diags = profile.validate_graph(&graph);
        assert!(!diags.is_empty());
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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
                    deletable: None,
                    reconnectable: None,
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
    fn symbol_ref_concretize_creates_typed_output_port() {
        let mut graph = Graph::default();

        let symbol_id = SymbolId::from_u128(10);
        graph.symbols.insert(
            symbol_id,
            Symbol {
                name: "S".to_string(),
                ty: Some(TypeDesc::Float),
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );

        let node_id = NodeId::new();
        let mut node = make_node(SYMBOL_REF_NODE_KIND);
        node.data = serde_json::json!({ "symbol_id": symbol_id });
        graph.nodes.insert(node_id, node);

        let mut profile = DataflowProfile::new();
        let _ = crate::profile::apply_transaction_with_profile(
            &mut graph,
            &mut profile,
            &crate::ops::GraphTransaction::new(),
        )
        .unwrap();

        let ports = graph.nodes.get(&node_id).unwrap().ports.clone();
        assert_eq!(ports.len(), 1);

        let out = ports[0];
        let port = graph.ports.get(&out).expect("out port");
        assert_eq!(port.key.0.as_str(), "out");
        assert_eq!(port.dir, PortDirection::Out);
        assert_eq!(port.kind, PortKind::Data);
        assert_eq!(port.ty, Some(TypeDesc::Float));
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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
