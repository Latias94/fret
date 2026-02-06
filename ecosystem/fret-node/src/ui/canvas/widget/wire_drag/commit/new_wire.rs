use super::prelude::*;

pub(super) fn commit_new_wire<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
    bounds: Rect,
    pos: Point,
    from: PortId,
    bundle: Vec<PortId>,
    target: Option<PortId>,
) -> CommitEmit {
    let window = cx.window();
    let mut connect_end_outcome = ConnectEndOutcome::NoOp;
    let mut connect_end_target = target;

    let suspended_pos = pos;
    if let Some(target) = target {
        connect_end_target = Some(target);
        enum Outcome {
            Apply(Vec<GraphOp>),
            Reject(DiagnosticSeverity, Arc<str>),
            Ignore,
            OpenConversionPicker(Vec<InsertNodeCandidate>),
        }

        let convert_at = crate::core::CanvasPoint {
            x: pos.x.0,
            y: pos.y.0,
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
                                            if insert_plan.decision == ConnectDecision::Accept {
                                                let tx = GraphTransaction {
                                                    label: None,
                                                    ops: insert_plan.ops.clone(),
                                                };
                                                let _ = apply_transaction(&mut scratch, &tx);
                                                ops_all.extend(insert_plan.ops);
                                                continue;
                                            }
                                        }
                                    }
                                }
                                if toast.is_none() {
                                    toast = NodeGraphCanvasWith::<M>::toast_from_diagnostics(
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

                let rows = crate::ui::canvas::searcher::build_rows_flat(&candidates, "");
                let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
                let origin = canvas.clamp_searcher_origin(
                    Point::new(Px(convert_at.x), Px(convert_at.y)),
                    visible,
                    bounds,
                    snapshot,
                );
                let active_row = NodeGraphCanvasWith::<M>::searcher_first_selectable_row(&rows)
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
                    this.hit_edge(graph, snapshot, &mut ctx, pos)
                })
                .ok()
                .flatten()
        };

        if let Some(edge_id) = hit_edge {
            connect_end_outcome = ConnectEndOutcome::OpenInsertNodePicker;
            canvas.open_edge_insert_node_picker(cx.host(), window, edge_id, pos);
        } else {
            let at = crate::core::CanvasPoint {
                x: pos.x.0,
                y: pos.y.0,
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

    CommitEmit {
        target: connect_end_target,
        outcome: connect_end_outcome,
    }
}
