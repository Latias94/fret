use std::sync::Arc;

use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use crate::core::{EdgeId, PortId};
use crate::ops::GraphOp;
use crate::rules::{ConnectDecision, DiagnosticSeverity};

use super::super::state::{ViewSnapshot, WireDragKind};
use super::NodeGraphCanvas;

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left
        || !canvas.interaction.sticky_wire
        || canvas.interaction.wire_drag.is_none()
    {
        return false;
    }

    let Some(mut w) = canvas.interaction.wire_drag.take() else {
        canvas.interaction.sticky_wire = false;
        canvas.interaction.sticky_wire_ignore_next_up = false;
        return true;
    };

    let from = match &w.kind {
        WireDragKind::New { from, .. } => *from,
        _ => {
            canvas.interaction.wire_drag = Some(w);
            return true;
        }
    };

    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let mut scratch_ports: Vec<PortId> = Vec::new();
    let hit_port = canvas.hit_port(
        geom.as_ref(),
        index.as_ref(),
        position,
        zoom,
        &mut scratch_ports,
    );

    if let Some(target) = hit_port {
        enum Outcome {
            Apply(Vec<GraphOp>),
            Reject(DiagnosticSeverity, Arc<str>),
            Ignore,
        }

        let outcome = {
            let presenter = &mut *canvas.presenter;
            canvas
                .graph
                .read_ref(cx.app, |graph| {
                    let plan = presenter.plan_connect(graph, from, target);
                    match plan.decision {
                        ConnectDecision::Accept => Outcome::Apply(plan.ops),
                        ConnectDecision::Reject => {
                            NodeGraphCanvas::toast_from_diagnostics(&plan.diagnostics)
                                .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                .unwrap_or(Outcome::Ignore)
                        }
                    }
                })
                .ok()
                .unwrap_or(Outcome::Ignore)
        };

        match outcome {
            Outcome::Apply(ops) => {
                canvas.apply_ops(cx.app, cx.window, ops);
                canvas.interaction.sticky_wire = false;
                canvas.interaction.sticky_wire_ignore_next_up = false;
                cx.release_pointer_capture();
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
                return true;
            }
            Outcome::Reject(sev, msg) => {
                canvas.show_toast(cx.app, cx.window, sev, msg);
            }
            Outcome::Ignore => {}
        }

        w.pos = position;
        canvas.interaction.wire_drag = Some(w);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    let at = canvas.interaction.last_canvas_pos.unwrap_or_default();
    let on_background = {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let order = this.node_order(graph, snapshot);
                let on_node = this.hit_node(graph, position, &order, zoom).is_some();
                if on_node {
                    return false;
                }
                let mut scratch_edges: Vec<EdgeId> = Vec::new();
                let on_edge = this
                    .hit_edge(
                        graph,
                        snapshot,
                        geom.as_ref(),
                        index.as_ref(),
                        position,
                        zoom,
                        &mut scratch_edges,
                    )
                    .is_some();
                !on_edge
            })
            .ok()
            .unwrap_or(false)
    };

    canvas.interaction.sticky_wire = false;
    canvas.interaction.sticky_wire_ignore_next_up = false;
    cx.release_pointer_capture();

    if on_background {
        canvas.open_connection_insert_node_picker(cx.app, from, at);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    false
}
