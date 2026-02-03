use std::sync::Arc;

use fret_core::{AppWindowId, Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::PortId;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::runtime::callbacks::ConnectEndOutcome;
use crate::ui::presenter::InsertNodeCandidate;

use super::super::conversion;
use super::super::searcher::SEARCHER_MAX_VISIBLE_ROWS;
use super::super::state::{
    ContextMenuTarget, LastConversionContext, SearcherState, ViewSnapshot, WireDrag, WireDragKind,
};
use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

fn severity_rank(sev: DiagnosticSeverity) -> u8 {
    match sev {
        DiagnosticSeverity::Info => 0,
        DiagnosticSeverity::Warning => 1,
        DiagnosticSeverity::Error => 2,
    }
}

pub(super) trait WireCommitCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn bounds(&self, last_bounds: Option<Rect>) -> Rect;
    fn release_pointer_capture(&mut self);
    fn request_redraw(&mut self);
    fn invalidate_paint(&mut self);
}

impl<'a, H: UiHost> WireCommitCx<H> for fret_ui::retained_bridge::EventCx<'a, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self, _last_bounds: Option<Rect>) -> Rect {
        self.bounds
    }

    fn release_pointer_capture(&mut self) {
        self.release_pointer_capture();
    }

    fn request_redraw(&mut self) {
        self.request_redraw();
    }

    fn invalidate_paint(&mut self) {
        self.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}

impl<'a, H: UiHost> WireCommitCx<H> for fret_ui::retained_bridge::CommandCx<'a, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self, last_bounds: Option<Rect>) -> Rect {
        last_bounds.unwrap_or_default()
    }

    fn release_pointer_capture(&mut self) {}

    fn request_redraw(&mut self) {
        self.request_redraw();
    }

    fn invalidate_paint(&mut self) {
        self.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}

pub(super) fn handle_wire_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
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

pub(super) fn handle_wire_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    handle_wire_left_up_with_forced_target(canvas, cx, snapshot, zoom, None)
}

pub(super) fn handle_wire_left_up_with_forced_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
    forced_target: Option<PortId>,
) -> bool {
    let Some(w) = canvas.interaction.wire_drag.take() else {
        return false;
    };
    let kind_for_callbacks = w.kind.clone();

    let window = cx.window();
    let bounds = cx.bounds(canvas.interaction.last_bounds);

    let (from_port, require_from_connectable_start) = match &w.kind {
        WireDragKind::New { from, .. } => (Some(*from), true),
        WireDragKind::Reconnect { fixed, .. } => (Some(*fixed), false),
        WireDragKind::ReconnectMany { edges } => (edges.first().map(|e| e.2), false),
    };

    let from_port_connectable = from_port
        .map(|port| {
            if !require_from_connectable_start {
                return true;
            }
            canvas
                .graph
                .read_ref(cx.host(), |graph| {
                    NodeGraphCanvasWith::<M>::port_is_connectable_start(
                        graph,
                        &snapshot.interaction,
                        port,
                    )
                })
                .ok()
                .unwrap_or(false)
        })
        .unwrap_or(false);
    let forced_target = forced_target.filter(|port| {
        canvas
            .graph
            .read_ref(cx.host(), |graph| {
                NodeGraphCanvasWith::<M>::port_is_connectable_end(
                    graph,
                    &snapshot.interaction,
                    *port,
                )
            })
            .ok()
            .unwrap_or(false)
    });
    let target = forced_target.or_else(|| {
        from_port.and_then(|from_port| {
            if !from_port_connectable {
                return None;
            }
            let (geom, index) = canvas.canvas_derived(&*cx.host(), snapshot);
            let this = &*canvas;
            let index = index.clone();
            this.graph
                .read_ref(cx.host(), |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                    this.pick_target_port(
                        graph,
                        snapshot,
                        &mut ctx,
                        from_port,
                        require_from_connectable_start,
                        w.pos,
                    )
                })
                .ok()
                .flatten()
        })
    });
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_port_diagnostic = None;

    let mut connect_end_outcome = ConnectEndOutcome::NoOp;
    let mut connect_end_target = target;

    match w.kind {
        WireDragKind::New { from, bundle } => {
            let suspended_pos = w.pos;
            if let Some(target) = target {
                connect_end_target = Some(target);
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                    OpenConversionPicker(Vec<InsertNodeCandidate>),
                }

                let convert_at = crate::core::CanvasPoint {
                    x: w.pos.x.0,
                    y: w.pos.y.0,
                };
                let (outcome, toast) = {
                    let presenter = &mut *canvas.presenter;
                    let style = canvas.style.clone();
                    canvas
                        .graph
                        .read_ref(cx.host(), |graph| {
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
                                let plan = presenter.plan_connect(
                                    &scratch,
                                    src,
                                    target,
                                    snapshot.interaction.connection_mode,
                                );
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
                                            toast =
                                                NodeGraphCanvasWith::<M>::toast_from_diagnostics(
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
                        canvas.apply_ops(cx.host(), window, ops);
                        connect_end_outcome = ConnectEndOutcome::Committed;
                        if let Some((sev, msg)) = toast {
                            canvas.show_toast(cx.host(), window, sev, msg);
                        }
                    }
                    Outcome::OpenConversionPicker(candidates) => {
                        connect_end_outcome = ConnectEndOutcome::OpenConversionPicker;
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
                            bounds,
                            snapshot,
                        );
                        let active_row =
                            NodeGraphCanvasWith::<M>::searcher_first_selectable_row(&rows)
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
                        connect_end_outcome = ConnectEndOutcome::Rejected;
                        canvas.show_toast(cx.host(), window, sev, msg);
                    }
                    Outcome::Ignore => {}
                }
            } else if bundle.is_empty() {
                let hit_edge = {
                    let (geom, index) = canvas.canvas_derived(&*cx.host(), snapshot);
                    let this = &*canvas;
                    let index = index.clone();
                    this.graph
                        .read_ref(cx.host(), |graph| {
                            let mut scratch = HitTestScratch::default();
                            let mut ctx =
                                HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                            this.hit_edge(graph, snapshot, &mut ctx, w.pos)
                        })
                        .ok()
                        .flatten()
                };

                if let Some(edge_id) = hit_edge {
                    connect_end_outcome = ConnectEndOutcome::OpenInsertNodePicker;
                    canvas.open_edge_insert_node_picker(cx.host(), window, edge_id, w.pos);
                } else {
                    let at = crate::core::CanvasPoint {
                        x: w.pos.x.0,
                        y: w.pos.y.0,
                    };
                    canvas.interaction.suspended_wire_drag = Some(WireDrag {
                        kind: WireDragKind::New {
                            from,
                            bundle: Vec::new(),
                        },
                        pos: suspended_pos,
                    });
                    connect_end_outcome = ConnectEndOutcome::OpenInsertNodePicker;
                    canvas.open_connection_insert_node_picker(cx.host(), from, at);
                }
            }
        }
        WireDragKind::Reconnect {
            edge,
            endpoint,
            fixed: _fixed,
        } => {
            if let Some(target) = target {
                connect_end_target = Some(target);
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let outcome = {
                    let presenter = &mut *canvas.presenter;
                    canvas
                        .graph
                        .read_ref(cx.host(), |graph| {
                            let plan = presenter.plan_reconnect_edge(
                                graph,
                                edge,
                                endpoint,
                                target,
                                snapshot.interaction.connection_mode,
                            );
                            match plan.decision {
                                ConnectDecision::Accept => Outcome::Apply(plan.ops),
                                ConnectDecision::Reject => {
                                    NodeGraphCanvasWith::<M>::toast_from_diagnostics(
                                        &plan.diagnostics,
                                    )
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
                        canvas.apply_ops(cx.host(), window, ops);
                        connect_end_outcome = ConnectEndOutcome::Committed;
                    }
                    Outcome::Reject(sev, msg) => {
                        connect_end_outcome = ConnectEndOutcome::Rejected;
                        canvas.show_toast(cx.host(), window, sev, msg);
                    }
                    Outcome::Ignore => {}
                }
            } else if snapshot.interaction.reconnect_on_drop_empty {
                let ops = canvas
                    .graph
                    .read_ref(cx.host(), |graph| {
                        let Some(edge_value) = graph.edges.get(&edge) else {
                            return Vec::new();
                        };
                        vec![GraphOp::RemoveEdge {
                            id: edge,
                            edge: edge_value.clone(),
                        }]
                    })
                    .ok()
                    .unwrap_or_default();
                if !ops.is_empty() {
                    let _ = canvas.commit_ops(cx.host(), window, Some("Disconnect Edge"), ops);
                    connect_end_outcome = ConnectEndOutcome::Committed;
                }
            }
        }
        WireDragKind::ReconnectMany { edges } => {
            if let Some(target) = target {
                connect_end_target = Some(target);
                let presenter = &mut *canvas.presenter;
                let (ops_all, toast) = canvas
                    .graph
                    .read_ref(cx.host(), |graph| {
                        let mut scratch = graph.clone();
                        let mut ops_all: Vec<GraphOp> = Vec::new();
                        let mut toast: Option<(DiagnosticSeverity, Arc<str>)> = None;

                        for (edge, endpoint, _fixed) in edges {
                            let plan = presenter.plan_reconnect_edge(
                                &scratch,
                                edge,
                                endpoint,
                                target,
                                snapshot.interaction.connection_mode,
                            );
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
                                        toast = NodeGraphCanvasWith::<M>::toast_from_diagnostics(
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
                    canvas.apply_ops(cx.host(), window, ops_all);
                    connect_end_outcome = ConnectEndOutcome::Committed;
                }
                if let Some((sev, msg)) = toast {
                    canvas.show_toast(cx.host(), window, sev, msg);
                }
            } else if snapshot.interaction.reconnect_on_drop_empty {
                let ops_all = canvas
                    .graph
                    .read_ref(cx.host(), |graph| {
                        let mut out: Vec<GraphOp> = Vec::new();
                        out.reserve(edges.len());
                        for (edge_id, _endpoint, _fixed) in edges {
                            let Some(edge_value) = graph.edges.get(&edge_id) else {
                                continue;
                            };
                            out.push(GraphOp::RemoveEdge {
                                id: edge_id,
                                edge: edge_value.clone(),
                            });
                        }
                        out
                    })
                    .ok()
                    .unwrap_or_default();

                if !ops_all.is_empty() {
                    let label = if ops_all.len() == 1 {
                        "Disconnect Edge"
                    } else {
                        "Disconnect Edges"
                    };
                    let _ = canvas.commit_ops(cx.host(), window, Some(label), ops_all);
                    connect_end_outcome = ConnectEndOutcome::Committed;
                }
            }
        }
    }

    canvas.emit_connect_end(
        snapshot.interaction.connection_mode,
        &kind_for_callbacks,
        connect_end_target,
        connect_end_outcome,
    );

    cx.release_pointer_capture();
    cx.request_redraw();
    cx.invalidate_paint();
    true
}
