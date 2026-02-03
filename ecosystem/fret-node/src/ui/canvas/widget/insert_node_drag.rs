use std::sync::Arc;

use fret_core::{InternalDragEvent, InternalDragKind, MouseButtons, Point, Rect};
use fret_runtime::DragKindId;
use fret_ui::UiHost;
use fret_ui_kit::dnd as ui_dnd;
use ui_dnd::{ActivationConstraint, AutoScrollConfig, CollisionStrategy, DndItemId, SensorOutput};

use crate::REROUTE_KIND;
use crate::core::{CanvasPoint, EdgeId};
use crate::ops::GraphOp;
use crate::rules::ConnectDecision;
use crate::ui::presenter::InsertNodeCandidate;

use super::super::state::{InsertNodeDragPreview, ViewSnapshot};
use super::{HitTestCtx, HitTestScratch};
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

/// Payload type for "drag a node from the palette/searcher into the canvas".
#[derive(Debug, Clone)]
pub(super) struct InsertNodeDragPayload {
    pub(super) candidate: InsertNodeCandidate,
}

pub(super) const DRAG_KIND_INSERT_NODE: DragKindId = DragKindId(0x4E4F44455F494E53);
const DND_DROP_CANVAS: DndItemId = DndItemId(0x4E4F44455F43414E);

fn canvas_to_window<M: NodeGraphCanvasMiddleware>(
    bounds: Rect,
    pos: Point,
    pan: CanvasPoint,
    zoom: f32,
) -> Point {
    let viewport = NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(bounds, pan, zoom);
    viewport.canvas_to_screen(pos)
}

pub(super) fn handle_pending_insert_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: MouseButtons,
    zoom: f32,
) -> bool {
    let Some(pending) = canvas.interaction.pending_insert_node_drag.clone() else {
        return false;
    };

    let Some(pointer_id) = cx.pointer_id else {
        canvas.interaction.pending_insert_node_drag = None;
        cx.release_pointer_capture();
        return false;
    };
    if pending.pointer_id != pointer_id {
        return false;
    }

    if !buttons.left {
        canvas.interaction.pending_insert_node_drag = None;
        if let Some(window) = cx.window {
            let dnd = ui_dnd::dnd_service_model_global(cx.app);
            ui_dnd::clear_pointer(
                cx.app.models_mut(),
                &dnd,
                window,
                DRAG_KIND_INSERT_NODE,
                pointer_id,
            );
        }
        cx.release_pointer_capture();
        return false;
    }

    if cx.window.is_none() {
        // Can't start an internal drag without a window id.
        canvas.interaction.pending_insert_node_drag = None;
        cx.release_pointer_capture();
        return false;
    }

    let Some(window) = cx.window else {
        return false;
    };
    let start_window = canvas_to_window::<M>(cx.bounds, pending.start_pos, snapshot.pan, zoom);
    let current_window = canvas_to_window::<M>(cx.bounds, position, snapshot.pan, zoom);

    let dnd = ui_dnd::dnd_service_model_global(cx.app);
    let frame_id = cx.app.frame_id();
    let tick_id = cx.app.tick_id();

    ui_dnd::register_droppable_rect(
        cx.app.models_mut(),
        &dnd,
        window,
        frame_id,
        DND_DROP_CANVAS,
        cx.bounds,
        0,
        false,
    );
    let update = ui_dnd::handle_pointer_move_or_init_in_scope(
        cx.app.models_mut(),
        &dnd,
        window,
        frame_id,
        DRAG_KIND_INSERT_NODE,
        ui_dnd::DND_SCOPE_DEFAULT,
        pointer_id,
        pending.start_tick,
        start_window,
        current_window,
        tick_id,
        ActivationConstraint::Distance { px: 6.0 },
        CollisionStrategy::PointerWithin,
        Some((cx.bounds, AutoScrollConfig::default())),
    );
    if !matches!(update.sensor, SensorOutput::DragStart { .. }) {
        return false;
    }

    cx.app.begin_cross_window_drag_with_kind(
        pointer_id,
        DRAG_KIND_INSERT_NODE,
        window,
        start_window,
        InsertNodeDragPayload {
            candidate: pending.candidate.clone(),
        },
    );
    ui_dnd::clear_pointer(
        cx.app.models_mut(),
        &dnd,
        window,
        DRAG_KIND_INSERT_NODE,
        pointer_id,
    );
    if let Some(drag) = cx.app.drag_mut(pointer_id)
        && drag.payload::<InsertNodeDragPayload>().is_some()
    {
        drag.dragging = true;
    }

    canvas.interaction.searcher = None;
    canvas.interaction.pending_insert_node_drag = None;
    cx.release_pointer_capture();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_internal_drag_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    event: &InternalDragEvent,
    zoom: f32,
) -> bool {
    let pointer_id = event.pointer_id;
    let payload = cx
        .app
        .drag(pointer_id)
        .and_then(|d| d.payload::<InsertNodeDragPayload>())
        .cloned();
    let Some(payload) = payload else {
        if canvas.interaction.insert_node_drag_preview.take().is_some() {
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        }
        return false;
    };

    match event.kind {
        InternalDragKind::Enter | InternalDragKind::Over => {
            let pos = event.position;
            let at = CanvasPoint {
                x: pos.x.0,
                y: pos.y.0,
            };

            let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
            let edge_hit: Option<EdgeId> = canvas
                .graph
                .read_ref(cx.app, |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                    canvas.hit_edge(graph, snapshot, &mut ctx, pos)
                })
                .ok()
                .flatten();

            let can_split_edge: Option<EdgeId> = edge_hit.and_then(|edge_id| {
                let candidate = &payload.candidate;
                let at = if candidate.kind.0 == REROUTE_KIND {
                    canvas.reroute_pos_for_invoked_at(pos)
                } else {
                    at
                };
                canvas
                    .graph
                    .read_ref(cx.app, |graph| {
                        let presenter = &mut *canvas.presenter;
                        let plan =
                            presenter.plan_split_edge_candidate(graph, edge_id, candidate, at);
                        matches!(plan.decision, ConnectDecision::Accept).then_some(edge_id)
                    })
                    .ok()
                    .flatten()
            });

            canvas.interaction.insert_node_drag_preview = Some(InsertNodeDragPreview {
                label: payload.candidate.label.clone(),
                pos,
                edge: can_split_edge,
            });

            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            cx.stop_propagation();
            return true;
        }
        InternalDragKind::Leave | InternalDragKind::Cancel => {
            if canvas.interaction.insert_node_drag_preview.take().is_some() {
                cx.request_redraw();
                cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            }
            cx.stop_propagation();
            return true;
        }
        InternalDragKind::Drop => {
            let pos = event.position;
            let at = CanvasPoint {
                x: pos.x.0,
                y: pos.y.0,
            };
            let candidate = payload.candidate.clone();
            canvas.record_recent_kind(&candidate.kind);

            let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
            let edge_hit: Option<EdgeId> = canvas
                .graph
                .read_ref(cx.app, |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                    canvas.hit_edge(graph, snapshot, &mut ctx, pos)
                })
                .ok()
                .flatten();

            let mut applied = false;

            if let Some(edge_id) = edge_hit {
                let at = if candidate.kind.0 == REROUTE_KIND {
                    canvas.reroute_pos_for_invoked_at(pos)
                } else {
                    at
                };
                let planned = canvas
                    .graph
                    .read_ref(cx.app, |graph| {
                        let presenter = &mut *canvas.presenter;
                        let plan =
                            presenter.plan_split_edge_candidate(graph, edge_id, &candidate, at);
                        match plan.decision {
                            ConnectDecision::Accept => Some(Ok(plan.ops)),
                            ConnectDecision::Reject => Some(Err(plan.diagnostics)),
                        }
                    })
                    .ok()
                    .flatten();

                if let Some(Ok(ops)) = planned {
                    let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
                    applied = canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops);
                    if applied && let Some(node_id) = node_id {
                        canvas.update_view_state(cx.app, |s| {
                            s.selected_edges.clear();
                            s.selected_groups.clear();
                            s.selected_nodes.clear();
                            s.selected_nodes.push(node_id);
                            s.draw_order.retain(|id| *id != node_id);
                            s.draw_order.push(node_id);
                        });
                    }
                } else if let Some(Err(diags)) = planned {
                    if let Some((sev, msg)) =
                        NodeGraphCanvasWith::<M>::toast_from_diagnostics(&diags)
                    {
                        canvas.show_toast(cx.app, cx.window, sev, msg);
                    }
                }
            }

            if !applied {
                let ops: Option<Vec<GraphOp>> = if candidate.kind.0 == REROUTE_KIND {
                    Some(NodeGraphCanvasWith::<M>::build_reroute_create_ops(at))
                } else {
                    let presenter = &mut *canvas.presenter;
                    canvas
                        .graph
                        .read_ref(cx.app, |graph| {
                            presenter.plan_create_node(graph, &candidate, at)
                        })
                        .ok()
                        .and_then(|r| r.ok())
                };

                if let Some(ops) = ops {
                    let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
                    if canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                        if let Some(node_id) = node_id {
                            canvas.update_view_state(cx.app, |s| {
                                s.selected_edges.clear();
                                s.selected_groups.clear();
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node_id);
                                s.draw_order.retain(|id| *id != node_id);
                                s.draw_order.push(node_id);
                            });
                        }
                    }
                } else {
                    canvas.show_toast(
                        cx.app,
                        cx.window,
                        crate::rules::DiagnosticSeverity::Info,
                        Arc::<str>::from("node insertion is not supported"),
                    );
                }
            }

            canvas.interaction.insert_node_drag_preview = None;
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            cx.stop_propagation();
            return true;
        }
    }
}
