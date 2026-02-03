use std::sync::Arc;

use fret_core::{Modifiers, Point, Px};
use fret_ui::UiHost;

use crate::rules::{ConnectDecision, DiagnosticSeverity};

use super::super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use super::diagnostics::severity_rank;
use crate::ui::canvas::conversion;
use crate::ui::canvas::state::{ViewSnapshot, WireDragKind};

pub(in super::super) fn handle_wire_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(mut w) = canvas.interaction.wire_drag.take() else {
        return false;
    };

    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let auto_pan_delta = (snapshot.interaction.auto_pan.on_connect)
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();
    w.pos = Point::new(
        Px(position.x.0 - auto_pan_delta.x),
        Px(position.y.0 - auto_pan_delta.y),
    );
    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    let pos = w.pos;

    if modifiers.shift {
        if let WireDragKind::New { from, bundle } = &mut w.kind {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
            let candidate = canvas.hit_port(&mut ctx, pos);

            if let Some(candidate) = candidate {
                let should_add = {
                    let this = &*canvas;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            if !NodeGraphCanvasWith::<M>::port_is_connectable_start(
                                graph,
                                &snapshot.interaction,
                                candidate,
                            ) {
                                return false;
                            }
                            NodeGraphCanvasWith::<M>::should_add_bundle_port(
                                graph, *from, bundle, candidate,
                            )
                        })
                        .ok()
                        .unwrap_or(false)
                };
                if should_add {
                    bundle.push(candidate);
                }
            }
        }
    }

    let (from_port, require_from_connectable_start) = match &w.kind {
        WireDragKind::New { from, .. } => (Some(*from), true),
        WireDragKind::Reconnect { fixed, .. } => (Some(*fixed), false),
        WireDragKind::ReconnectMany { edges } => (edges.first().map(|e| e.2), false),
    };

    let new_hover = if let Some(from_port) = from_port {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch = HitTestScratch::default();
                let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                this.pick_wire_hover_port(
                    graph,
                    snapshot,
                    &mut ctx,
                    from_port,
                    require_from_connectable_start,
                    pos,
                )
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let new_hover_edge = if new_hover.is_none() {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch = HitTestScratch::default();
                let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                this.hit_edge(graph, snapshot, &mut ctx, pos)
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let new_hover_valid = if let Some(target) = new_hover {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                if !NodeGraphCanvasWith::<M>::port_is_connectable_end(
                    graph,
                    &snapshot.interaction,
                    target,
                ) {
                    return (
                        false,
                        Some((
                            DiagnosticSeverity::Error,
                            Arc::<str>::from("Port is not connectable"),
                        )),
                    );
                }

                let mut best_diag: Option<(DiagnosticSeverity, Arc<str>)> = None;
                let mut accept = false;

                let mut consider_diag = |plan: &crate::rules::ConnectPlan| {
                    for d in plan.diagnostics.iter() {
                        let next = (d.severity, Arc::<str>::from(d.message.clone()));
                        match &best_diag {
                            Some((best_sev, _))
                                if severity_rank(*best_sev) > severity_rank(next.0) => {}
                            _ => best_diag = Some(next),
                        }
                    }
                };

                match &w.kind {
                    WireDragKind::New { from, bundle } => {
                        let sources = if bundle.is_empty() {
                            std::slice::from_ref(from)
                        } else {
                            bundle.as_slice()
                        };
                        for src in sources {
                            let plan = presenter.can_connect(
                                graph,
                                *src,
                                target,
                                snapshot.interaction.connection_mode,
                            );
                            if plan.decision == ConnectDecision::Accept {
                                accept = true;
                                break;
                            }
                            consider_diag(&plan);
                        }
                    }
                    WireDragKind::Reconnect { edge, endpoint, .. } => {
                        let plan = presenter.can_reconnect_edge(
                            graph,
                            *edge,
                            *endpoint,
                            target,
                            snapshot.interaction.connection_mode,
                        );
                        if plan.decision == ConnectDecision::Accept {
                            accept = true;
                        } else {
                            consider_diag(&plan);
                        }
                    }
                    WireDragKind::ReconnectMany { edges } => {
                        for (edge, endpoint, _fixed) in edges {
                            let plan = presenter.can_reconnect_edge(
                                graph,
                                *edge,
                                *endpoint,
                                target,
                                snapshot.interaction.connection_mode,
                            );
                            if plan.decision == ConnectDecision::Accept {
                                accept = true;
                                break;
                            }
                            consider_diag(&plan);
                        }
                    }
                }

                if accept {
                    (true, None)
                } else {
                    let diag = best_diag.or_else(|| {
                        Some((
                            DiagnosticSeverity::Error,
                            Arc::<str>::from("Invalid connection"),
                        ))
                    });
                    (false, diag)
                }
            })
            .ok()
            .unwrap_or((false, None))
    } else {
        (false, None)
    };

    let (new_hover_valid, new_hover_diag) = new_hover_valid;

    let new_hover_convertible = if !new_hover_valid {
        if let Some(target) = new_hover {
            match &w.kind {
                WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                    let presenter = &mut *canvas.presenter;
                    canvas
                        .graph
                        .read_ref(cx.app, |graph| {
                            if !NodeGraphCanvasWith::<M>::port_is_connectable_end(
                                graph,
                                &snapshot.interaction,
                                target,
                            ) {
                                return false;
                            }
                            conversion::is_convertible(presenter, graph, *from, target)
                        })
                        .ok()
                        .unwrap_or(false)
                }
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    };

    if canvas.interaction.hover_port != new_hover
        || canvas.interaction.hover_port_valid != new_hover_valid
        || canvas.interaction.hover_port_convertible != new_hover_convertible
        || canvas.interaction.hover_port_diagnostic != new_hover_diag
    {
        canvas.interaction.hover_port = new_hover;
        canvas.interaction.hover_port_valid = new_hover_valid;
        canvas.interaction.hover_port_convertible = new_hover_convertible;
        canvas.interaction.hover_port_diagnostic = new_hover_diag;
    }

    canvas.interaction.hover_edge = new_hover_edge;
    canvas.interaction.wire_drag = Some(w);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
