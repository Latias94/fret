use super::*;

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
        .max_by_key(|diagnostic| match diagnostic.severity {
            DiagnosticSeverity::Info => 0,
            DiagnosticSeverity::Warning => 1,
            DiagnosticSeverity::Error => 2,
        })
        .map(|diagnostic| {
            (
                diagnostic.severity,
                Arc::<str>::from(diagnostic.message.clone()),
            )
        })
}

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
            .is_some_and(|port| port.dir == from_port.dir && port.kind == from_port.kind)
    });

    let mut best_accept: Option<Vec<GraphOp>> = None;
    let mut best_reject_toast: Option<(DiagnosticSeverity, Arc<str>)> = None;

    for &port_id in &node.ports {
        let Some(port) = scratch.ports.get(&port_id) else {
            continue;
        };
        if port.dir != desired_dir || port.kind != from_port.kind {
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
