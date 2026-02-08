use std::sync::Arc;

use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use crate::ops::GraphOp;
use crate::rules::{ConnectDecision, DiagnosticSeverity};

use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{ViewSnapshot, WireDragKind};

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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
    let mut scratch = HitTestScratch::default();
    let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
    let hit_port = canvas.hit_port(&mut ctx, position);

    let target = hit_port.filter(|target| {
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                NodeGraphCanvasWith::<M>::port_is_connectable_end(
                    graph,
                    &snapshot.interaction,
                    *target,
                )
            })
            .ok()
            .unwrap_or(false)
    });

    if let Some(target) = target {
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
                    let plan = presenter.plan_connect(
                        graph,
                        from,
                        target,
                        snapshot.interaction.connection_mode,
                    );
                    match plan.decision {
                        ConnectDecision::Accept => Outcome::Apply(plan.ops),
                        ConnectDecision::Reject => {
                            NodeGraphCanvasWith::<M>::toast_from_diagnostics(&plan.diagnostics)
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
    let (on_node, hit_edge) = {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let on_node = geom.order.iter().rev().any(|id| {
                    geom.nodes
                        .get(id)
                        .is_some_and(|ng| ng.rect.contains(position))
                });
                if on_node {
                    return (true, None);
                }
                let mut scratch = HitTestScratch::default();
                let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                let hit_edge = this.hit_edge(graph, snapshot, &mut ctx, position);
                (false, hit_edge)
            })
            .ok()
            .unwrap_or((false, None))
    };

    canvas.interaction.sticky_wire = false;
    canvas.interaction.sticky_wire_ignore_next_up = false;
    cx.release_pointer_capture();

    if on_node {
        return false;
    }

    if let Some(edge_id) = hit_edge {
        canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, position);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    // If we're not on a node or edge, open the insert picker for the current wire.
    canvas.open_connection_insert_node_picker(cx.app, from, at);
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    return true;

    // unreachable
}
