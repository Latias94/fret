use std::sync::Arc;

use fret_core::{Modifiers, Point, Px};
use fret_ui::UiHost;

use crate::core::{EdgeId, PortId};
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::ui::presenter::InsertNodeCandidate;

use super::super::conversion;
use super::super::searcher::SEARCHER_MAX_VISIBLE_ROWS;
use super::super::state::{
    ContextMenuTarget, LastConversionContext, SearcherState, ViewSnapshot, WireDrag, WireDragKind,
};
use super::NodeGraphCanvas;

pub(super) fn handle_wire_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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
        .then(|| NodeGraphCanvas::auto_pan_delta(snapshot, position, cx.bounds))
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
            let mut scratch_ports: Vec<PortId> = Vec::new();
            let candidate =
                canvas.hit_port(geom.as_ref(), index.as_ref(), pos, zoom, &mut scratch_ports);

            if let Some(candidate) = candidate {
                let should_add = {
                    let this = &*canvas;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            NodeGraphCanvas::should_add_bundle_port(graph, *from, bundle, candidate)
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

    let from_port = match &w.kind {
        WireDragKind::New { from, .. } => Some(*from),
        WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
        WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
    };

    let new_hover = if let Some(from_port) = from_port {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch_ports: Vec<PortId> = Vec::new();
                this.pick_target_port(
                    graph,
                    snapshot,
                    geom.as_ref(),
                    index.as_ref(),
                    from_port,
                    pos,
                    zoom,
                    &mut scratch_ports,
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
                let mut scratch_edges: Vec<EdgeId> = Vec::new();
                this.hit_edge(
                    graph,
                    snapshot,
                    geom.as_ref(),
                    index.as_ref(),
                    pos,
                    zoom,
                    &mut scratch_edges,
                )
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
                let mut scratch = graph.clone();
                match &w.kind {
                    WireDragKind::New { from, bundle } => {
                        let sources = if bundle.is_empty() {
                            std::slice::from_ref(from)
                        } else {
                            bundle.as_slice()
                        };
                        let mut any_accept = false;
                        for src in sources {
                            let plan = presenter.plan_connect(&scratch, *src, target);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                    WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
                        presenter
                            .plan_reconnect_edge(&scratch, *edge, *endpoint, target)
                            .decision,
                        ConnectDecision::Accept
                    ),
                    WireDragKind::ReconnectMany { edges } => {
                        let mut any_accept = false;
                        for (edge, endpoint, _fixed) in edges {
                            let plan =
                                presenter.plan_reconnect_edge(&scratch, *edge, *endpoint, target);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                }
            })
            .ok()
            .unwrap_or(false)
    } else {
        false
    };

    let new_hover_convertible = if !new_hover_valid {
        if let Some(target) = new_hover {
            match &w.kind {
                WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                    let presenter = &mut *canvas.presenter;
                    canvas
                        .graph
                        .read_ref(cx.app, |graph| {
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
    {
        canvas.interaction.hover_port = new_hover;
        canvas.interaction.hover_port_valid = new_hover_valid;
        canvas.interaction.hover_port_convertible = new_hover_convertible;
    }

    canvas.interaction.hover_edge = new_hover_edge;
    canvas.interaction.wire_drag = Some(w);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_wire_left_up<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    let Some(w) = canvas.interaction.wire_drag.take() else {
        return false;
    };

    let from_port = match &w.kind {
        WireDragKind::New { from, .. } => Some(*from),
        WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
        WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
    };
    let target = from_port.and_then(|from_port| {
        let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
        let this = &*canvas;
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch_ports: Vec<PortId> = Vec::new();
                this.pick_target_port(
                    graph,
                    snapshot,
                    geom.as_ref(),
                    index.as_ref(),
                    from_port,
                    w.pos,
                    zoom,
                    &mut scratch_ports,
                )
            })
            .ok()
            .flatten()
    });
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;

    match w.kind {
        WireDragKind::New { from, bundle } => {
            let suspended_pos = w.pos;
            if let Some(target) = target {
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                    OpenConversionPicker(Vec<InsertNodeCandidate>),
                }

                let convert_at =
                    NodeGraphCanvas::screen_to_canvas(cx.bounds, w.pos, snapshot.pan, zoom);
                let (outcome, toast) = {
                    let presenter = &mut *canvas.presenter;
                    let style = canvas.style.clone();
                    canvas
                        .graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch = graph.clone();
                            let sources: Vec<PortId> = if bundle.is_empty() {
                                vec![from]
                            } else {
                                bundle
                            };
                            let allow_convert = sources.len() == 1;
                            let mut picker: Option<Vec<InsertNodeCandidate>> = None;
                            let mut ops_all: Vec<GraphOp> = Vec::new();
                            let mut toast: Option<(DiagnosticSeverity, Arc<str>)> = None;

                            for src in sources {
                                let plan = presenter.plan_connect(&scratch, src, target);
                                match plan.decision {
                                    ConnectDecision::Accept => {
                                        let tx = GraphTransaction {
                                            label: None,
                                            ops: plan.ops.clone(),
                                        };
                                        let _ = apply_transaction(&mut scratch, &tx);
                                        ops_all.extend(plan.ops);
                                    }
                                    ConnectDecision::Reject => {
                                        if allow_convert {
                                            let conversions =
                                                presenter.list_conversions(&scratch, src, target);
                                            if conversions.len() > 1 {
                                                picker = Some(conversion::build_picker_candidates(
                                                    presenter,
                                                    &scratch,
                                                    src,
                                                    target,
                                                    conversions,
                                                ));
                                                break;
                                            }
                                            if conversions.len() == 1 {
                                                if let Some(insert_plan) =
                                                    conversion::try_auto_insert_conversion(
                                                        presenter,
                                                        &scratch,
                                                        &style,
                                                        zoom,
                                                        src,
                                                        target,
                                                        convert_at,
                                                        &conversions,
                                                    )
                                                {
                                                    if insert_plan.decision
                                                        == ConnectDecision::Accept
                                                    {
                                                        let tx = GraphTransaction {
                                                            label: None,
                                                            ops: insert_plan.ops.clone(),
                                                        };
                                                        let _ =
                                                            apply_transaction(&mut scratch, &tx);
                                                        ops_all.extend(insert_plan.ops);
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                        if toast.is_none() {
                                            toast = NodeGraphCanvas::toast_from_diagnostics(
                                                &plan.diagnostics,
                                            );
                                        }
                                    }
                                }
                            }

                            let outcome = if let Some(picker) = picker {
                                Outcome::OpenConversionPicker(picker)
                            } else if ops_all.is_empty() {
                                if let Some((sev, msg)) = toast.clone() {
                                    Outcome::Reject(sev, msg)
                                } else {
                                    Outcome::Ignore
                                }
                            } else {
                                Outcome::Apply(ops_all)
                            };
                            (outcome, toast)
                        })
                        .ok()
                        .unwrap_or((Outcome::Ignore, None))
                };

                match outcome {
                    Outcome::Apply(ops) => {
                        canvas.apply_ops(cx.app, cx.window, ops);
                        if let Some((sev, msg)) = toast {
                            canvas.show_toast(cx.app, cx.window, sev, msg);
                        }
                    }
                    Outcome::OpenConversionPicker(candidates) => {
                        canvas.interaction.suspended_wire_drag = Some(WireDrag {
                            kind: WireDragKind::New {
                                from,
                                bundle: Vec::new(),
                            },
                            pos: suspended_pos,
                        });
                        canvas.interaction.last_conversion = Some(LastConversionContext {
                            from,
                            to: target,
                            at: convert_at,
                            candidates: candidates.clone(),
                        });

                        let rows = super::super::searcher::build_rows_flat(&candidates, "");
                        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
                        let origin = canvas.clamp_searcher_origin(
                            Point::new(Px(convert_at.x), Px(convert_at.y)),
                            visible,
                            cx.bounds,
                            snapshot,
                        );
                        let active_row = NodeGraphCanvas::searcher_first_selectable_row(&rows)
                            .min(rows.len().saturating_sub(1));

                        canvas.interaction.context_menu = None;
                        canvas.interaction.searcher = Some(SearcherState {
                            origin,
                            invoked_at: Point::new(Px(convert_at.x), Px(convert_at.y)),
                            target: ContextMenuTarget::ConnectionConvertPicker {
                                from,
                                to: target,
                                at: convert_at,
                            },
                            query: String::new(),
                            candidates,
                            recent_kinds: canvas.interaction.recent_kinds.clone(),
                            rows,
                            hovered_row: None,
                            active_row,
                            scroll: 0,
                        });
                    }
                    Outcome::Reject(sev, msg) => {
                        canvas.show_toast(cx.app, cx.window, sev, msg);
                    }
                    Outcome::Ignore => {}
                }
            } else if bundle.is_empty() {
                let hit_edge = {
                    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
                    let this = &*canvas;
                    let index = index.clone();
                    this.graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch_edges: Vec<EdgeId> = Vec::new();
                            this.hit_edge(
                                graph,
                                snapshot,
                                geom.as_ref(),
                                index.as_ref(),
                                w.pos,
                                zoom,
                                &mut scratch_edges,
                            )
                        })
                        .ok()
                        .flatten()
                };

                if let Some(edge_id) = hit_edge {
                    canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, w.pos);
                } else {
                    let at =
                        NodeGraphCanvas::screen_to_canvas(cx.bounds, w.pos, snapshot.pan, zoom);
                    canvas.interaction.suspended_wire_drag = Some(WireDrag {
                        kind: WireDragKind::New {
                            from,
                            bundle: Vec::new(),
                        },
                        pos: suspended_pos,
                    });
                    canvas.open_connection_insert_node_picker(cx.app, from, at);
                }
            }
        }
        WireDragKind::Reconnect {
            edge,
            endpoint,
            fixed: _fixed,
        } => {
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
                            let plan = presenter.plan_reconnect_edge(graph, edge, endpoint, target);
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
                    Outcome::Apply(ops) => canvas.apply_ops(cx.app, cx.window, ops),
                    Outcome::Reject(sev, msg) => {
                        canvas.show_toast(cx.app, cx.window, sev, msg);
                    }
                    Outcome::Ignore => {}
                }
            }
        }
        WireDragKind::ReconnectMany { edges } => {
            if let Some(target) = target {
                let presenter = &mut *canvas.presenter;
                let (ops_all, toast) = canvas
                    .graph
                    .read_ref(cx.app, |graph| {
                        let mut scratch = graph.clone();
                        let mut ops_all: Vec<GraphOp> = Vec::new();
                        let mut toast: Option<(DiagnosticSeverity, Arc<str>)> = None;

                        for (edge, endpoint, _fixed) in edges {
                            let plan =
                                presenter.plan_reconnect_edge(&scratch, edge, endpoint, target);
                            match plan.decision {
                                ConnectDecision::Accept => {
                                    let tx = GraphTransaction {
                                        label: None,
                                        ops: plan.ops.clone(),
                                    };
                                    let _ = apply_transaction(&mut scratch, &tx);
                                    ops_all.extend(plan.ops);
                                }
                                ConnectDecision::Reject => {
                                    if toast.is_none() {
                                        toast = NodeGraphCanvas::toast_from_diagnostics(
                                            &plan.diagnostics,
                                        );
                                    }
                                }
                            }
                        }

                        (ops_all, toast)
                    })
                    .ok()
                    .unwrap_or_default();

                if !ops_all.is_empty() {
                    canvas.apply_ops(cx.app, cx.window, ops_all);
                }
                if let Some((sev, msg)) = toast {
                    canvas.show_toast(cx.app, cx.window, sev, msg);
                }
            }
        }
    }

    cx.release_pointer_capture();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
