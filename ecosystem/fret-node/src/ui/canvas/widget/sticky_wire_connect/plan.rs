use std::sync::Arc;

use fret_ui::UiHost;

use crate::ops::GraphOp;
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::ui::canvas::widget::*;

pub(super) fn connectable_sticky_wire_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    hit_port: Option<crate::core::PortId>,
) -> Option<crate::core::PortId> {
    hit_port.filter(|target| {
        canvas
            .graph
            .read_ref(host, |graph| {
                NodeGraphCanvasWith::<M>::port_is_connectable_end(
                    graph,
                    &snapshot.interaction,
                    *target,
                )
            })
            .ok()
            .unwrap_or(false)
    })
}

pub(super) enum StickyWireConnectOutcome {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

pub(super) fn plan_sticky_wire_connect_outcome<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    from_port: crate::core::PortId,
    target_port: crate::core::PortId,
) -> StickyWireConnectOutcome {
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            let plan = presenter.plan_connect(
                graph,
                from_port,
                target_port,
                snapshot.interaction.connection_mode,
            );
            match plan.decision {
                ConnectDecision::Accept => StickyWireConnectOutcome::Apply(plan.ops),
                ConnectDecision::Reject => {
                    NodeGraphCanvasWith::<M>::toast_from_diagnostics(&plan.diagnostics)
                        .map(|(severity, message)| {
                            StickyWireConnectOutcome::Reject(severity, message)
                        })
                        .unwrap_or(StickyWireConnectOutcome::Ignore)
                }
            }
        })
        .ok()
        .unwrap_or(StickyWireConnectOutcome::Ignore)
}
