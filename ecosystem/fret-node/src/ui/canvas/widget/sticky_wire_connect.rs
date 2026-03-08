use std::sync::Arc;

use fret_core::Point;
use fret_ui::UiHost;

use crate::ops::GraphOp;
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::ui::canvas::state::WireDrag;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

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

pub(super) fn handle_sticky_wire_connect_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    from_port: crate::core::PortId,
    target_port: crate::core::PortId,
    wire_drag: &mut WireDrag,
    position: Point,
) -> bool {
    match plan_sticky_wire_connect_outcome(canvas, cx.app, snapshot, from_port, target_port) {
        StickyWireConnectOutcome::Apply(ops) => {
            canvas.apply_ops(cx.app, cx.window, ops);
            super::sticky_wire_targets::reset_sticky_wire_state(canvas);
            finish_sticky_wire_pointer_down(cx);
            true
        }
        StickyWireConnectOutcome::Reject(severity, message) => {
            canvas.show_toast(cx.app, cx.window, severity, message);
            wire_drag.pos = position;
            canvas.interaction.wire_drag = Some(wire_drag.clone());
            finish_sticky_wire_pointer_down(cx);
            true
        }
        StickyWireConnectOutcome::Ignore => {
            wire_drag.pos = position;
            canvas.interaction.wire_drag = Some(wire_drag.clone());
            finish_sticky_wire_pointer_down(cx);
            true
        }
    }
}

enum StickyWireConnectOutcome {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

fn plan_sticky_wire_connect_outcome<H: UiHost, M: NodeGraphCanvasMiddleware>(
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

fn finish_sticky_wire_pointer_down<H: UiHost>(cx: &mut fret_ui::retained_bridge::EventCx<'_, H>) {
    cx.release_pointer_capture();
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}
