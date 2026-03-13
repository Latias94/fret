use crate::core::Graph;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::ui::canvas::state::WireDragKind;
use crate::ui::presenter::NodeGraphPresenter;

use super::preflight::FocusedPortHintInput;
use crate::ui::canvas::widget::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct FocusedPortHintOutcome {
    pub valid: bool,
    pub convertible: bool,
}

pub(super) fn evaluate_focused_port_hints(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    input: &FocusedPortHintInput,
) -> FocusedPortHintOutcome {
    let mut scratch = graph.clone();
    let valid = evaluate_valid_hint(presenter, &mut scratch, input);
    let convertible = evaluate_convertible_hint(presenter, &scratch, input, valid);

    FocusedPortHintOutcome { valid, convertible }
}

fn evaluate_valid_hint(
    presenter: &mut dyn NodeGraphPresenter,
    scratch: &mut Graph,
    input: &FocusedPortHintInput,
) -> bool {
    match &input.wire_drag.kind {
        WireDragKind::New { from, bundle } => {
            let sources = if bundle.is_empty() {
                std::slice::from_ref(from)
            } else {
                bundle.as_slice()
            };
            let mut any_accept = false;
            for source in sources {
                let plan = presenter.plan_connect(scratch, *source, input.target, input.mode);
                if plan.decision != ConnectDecision::Accept {
                    continue;
                }
                any_accept = true;
                apply_plan_ops(scratch, plan.ops.clone());
            }
            any_accept
        }
        WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
            presenter
                .plan_reconnect_edge(scratch, *edge, *endpoint, input.target, input.mode)
                .decision,
            ConnectDecision::Accept
        ),
        WireDragKind::ReconnectMany { edges } => {
            let mut any_accept = false;
            for (edge, endpoint, _fixed) in edges {
                let plan = presenter.plan_reconnect_edge(
                    scratch,
                    *edge,
                    *endpoint,
                    input.target,
                    input.mode,
                );
                if plan.decision != ConnectDecision::Accept {
                    continue;
                }
                any_accept = true;
                apply_plan_ops(scratch, plan.ops.clone());
            }
            any_accept
        }
    }
}

fn evaluate_convertible_hint(
    presenter: &mut dyn NodeGraphPresenter,
    scratch: &Graph,
    input: &FocusedPortHintInput,
    valid: bool,
) -> bool {
    if valid {
        return false;
    }

    match &input.wire_drag.kind {
        WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
            super::super::conversion::is_convertible(presenter, scratch, *from, input.target)
        }
        _ => false,
    }
}

fn apply_plan_ops(graph: &mut Graph, ops: Vec<GraphOp>) {
    let tx = GraphTransaction { label: None, ops };
    let _ = apply_transaction(graph, &tx);
}
