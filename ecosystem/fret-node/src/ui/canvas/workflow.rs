//! Reusable editor workflows for the node graph canvas.
//!
//! This module is intentionally UI-light: it focuses on planning ops for common editor-grade
//! workflows (XyFlow/ImGui-node-editor parity), while letting the canvas decide how to surface UI
//! (context menus, searchers, overlays).

use std::sync::Arc;

use crate::core::{Graph, NodeId, PortDirection, PortId};
use crate::interaction::NodeGraphConnectionMode;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::ui::presenter::NodeGraphPresenter;

#[derive(Debug, Clone)]
pub(crate) struct WireDropInsertPlan {
    pub ops: Vec<GraphOp>,
    pub created_node: Option<NodeId>,
    pub continue_from: Option<PortId>,
    pub toast: Option<(DiagnosticSeverity, Arc<str>)>,
}

fn first_added_node_id(ops: &[GraphOp]) -> Option<NodeId> {
    for op in ops {
        if let GraphOp::AddNode { id, .. } = op {
            return Some(*id);
        }
    }
    None
}

fn toast_from_rejected_connect(
    plan: &crate::rules::ConnectPlan,
) -> Option<(DiagnosticSeverity, Arc<str>)> {
    plan.diagnostics
        .iter()
        .max_by_key(|d| match d.severity {
            DiagnosticSeverity::Info => 0,
            DiagnosticSeverity::Warning => 1,
            DiagnosticSeverity::Error => 2,
        })
        .map(|d| (d.severity, Arc::<str>::from(d.message.clone())))
}

/// Plans "wire drop → insert node → auto-connect" as a single op list.
///
/// Behavior:
/// - Always keeps `insert_ops` (so selecting a node from the picker always inserts something).
/// - Attempts to connect the dropped wire endpoint (`from`) to the first compatible port on the
///   created node, using presenter-driven `plan_connect`.
/// - If auto-connect fails, returns `toast` but still returns insertion ops.
pub(crate) fn plan_wire_drop_insert(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    from: PortId,
    mode: NodeGraphConnectionMode,
    insert_ops: Vec<GraphOp>,
) -> WireDropInsertPlan {
    let created_node = first_added_node_id(&insert_ops);
    let Some(created_node) = created_node else {
        return WireDropInsertPlan {
            ops: insert_ops,
            created_node: None,
            continue_from: None,
            toast: None,
        };
    };

    let mut scratch = graph.clone();
    let insert_tx = GraphTransaction {
        label: None,
        ops: insert_ops.clone(),
    };
    if let Err(err) = apply_transaction(&mut scratch, &insert_tx) {
        return WireDropInsertPlan {
            ops: Vec::new(),
            created_node: None,
            continue_from: None,
            toast: Some((DiagnosticSeverity::Error, Arc::<str>::from(err.to_string()))),
        };
    }

    let Some(from_port) = scratch.ports.get(&from) else {
        return WireDropInsertPlan {
            ops: insert_ops,
            created_node: Some(created_node),
            continue_from: None,
            toast: Some((
                DiagnosticSeverity::Error,
                Arc::<str>::from("missing wire source port"),
            )),
        };
    };

    let desired_dir = match from_port.dir {
        PortDirection::In => PortDirection::Out,
        PortDirection::Out => PortDirection::In,
    };

    let Some(node) = scratch.nodes.get(&created_node) else {
        return WireDropInsertPlan {
            ops: insert_ops,
            created_node: Some(created_node),
            continue_from: None,
            toast: Some((
                DiagnosticSeverity::Error,
                Arc::<str>::from("missing inserted node"),
            )),
        };
    };

    let continue_from = node.ports.iter().copied().find(|port_id| {
        scratch
            .ports
            .get(port_id)
            .is_some_and(|p| p.dir == from_port.dir && p.kind == from_port.kind)
    });

    let mut best_accept: Option<Vec<GraphOp>> = None;
    let mut best_reject_toast: Option<(DiagnosticSeverity, Arc<str>)> = None;

    for &port_id in &node.ports {
        let Some(p) = scratch.ports.get(&port_id) else {
            continue;
        };
        if p.dir != desired_dir {
            continue;
        }
        if p.kind != from_port.kind {
            continue;
        }

        let plan = presenter.plan_connect(&scratch, from, port_id, mode);
        match plan.decision {
            ConnectDecision::Accept => {
                best_accept = Some(plan.ops);
                break;
            }
            ConnectDecision::Reject => {
                if best_reject_toast.is_none() {
                    best_reject_toast = toast_from_rejected_connect(&plan);
                }
            }
        }
    }

    match best_accept {
        Some(connect_ops) => {
            let mut ops_all = insert_ops;
            ops_all.extend(connect_ops);
            WireDropInsertPlan {
                ops: ops_all,
                created_node: Some(created_node),
                continue_from,
                toast: None,
            }
        }
        None => WireDropInsertPlan {
            ops: insert_ops,
            created_node: Some(created_node),
            continue_from: None,
            toast: best_reject_toast.or_else(|| {
                Some((
                    DiagnosticSeverity::Info,
                    Arc::<str>::from("inserted node has no compatible port to auto-connect"),
                ))
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{CanvasPoint, Node, NodeKindKey, Port, PortCapacity, PortKey, PortKind};
    use crate::schema::{NodeRegistry, NodeSchema, PortDecl};
    use crate::types::TypeDesc;
    use crate::ui::presenter::{InsertNodeCandidate, RegistryNodeGraphPresenter};

    #[test]
    fn wire_drop_insert_autoconnects_first_compatible_port() {
        let mut reg = NodeRegistry::default();
        reg.register(NodeSchema {
            kind: NodeKindKey::new("demo.pipe"),
            latest_kind_version: 1,
            kind_aliases: vec![],
            title: "Pipe".to_string(),
            category: vec![],
            keywords: vec![],
            ports: vec![
                PortDecl {
                    key: PortKey::new("in"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: Some(TypeDesc::Float),
                    label: None,
                },
                PortDecl {
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    ty: Some(TypeDesc::Float),
                    label: None,
                },
            ],
            default_data: serde_json::Value::Null,
        });

        let mut graph = Graph::default();
        let src_node = NodeId::new();
        let src_out = PortId::new();
        graph.nodes.insert(
            src_node,
            Node {
                kind: NodeKindKey::new("demo.src"),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![src_out],
                data: serde_json::Value::Null,
            },
        );
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

        let mut presenter = RegistryNodeGraphPresenter::new(reg);
        let candidate = InsertNodeCandidate {
            kind: NodeKindKey::new("demo.pipe"),
            label: Arc::<str>::from("Pipe"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        };
        let insert_ops = presenter
            .plan_create_node(&graph, &candidate, CanvasPoint { x: 10.0, y: 20.0 })
            .expect("create ops");

        let planned = plan_wire_drop_insert(
            &mut presenter,
            &graph,
            src_out,
            crate::interaction::NodeGraphConnectionMode::Strict,
            insert_ops,
        );
        assert!(planned.toast.is_none());
        assert!(planned.created_node.is_some());
        assert!(planned.continue_from.is_some());
        assert!(!planned.ops.is_empty());

        let tx = GraphTransaction {
            label: None,
            ops: planned.ops,
        };
        apply_transaction(&mut graph, &tx).expect("apply");
        assert_eq!(graph.edges.len(), 1);
    }
}
