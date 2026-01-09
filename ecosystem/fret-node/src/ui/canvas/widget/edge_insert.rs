use std::sync::Arc;

use fret_core::{AppWindowId, Point};
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use crate::REROUTE_KIND;
use crate::core::{CanvasPoint, EdgeId, NodeKindKey};
use crate::ops::GraphOp;
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::ui::presenter::{
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
};

use super::super::searcher::{SEARCHER_MAX_VISIBLE_ROWS, SearcherRowKind};
use super::super::state::{ContextMenuState, ContextMenuTarget, SearcherState};
use super::NodeGraphCanvas;

pub(super) fn open_edge_insert_node_picker<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    host: &mut H,
    window: Option<AppWindowId>,
    edge: EdgeId,
    invoked_at: Point,
) {
    let candidates: Vec<InsertNodeCandidate> = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(host, |graph| {
                presenter.list_insertable_nodes_for_edge(graph, edge)
            })
            .ok()
            .unwrap_or_default()
    };

    let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
    menu_candidates.push(InsertNodeCandidate {
        kind: NodeKindKey::new(REROUTE_KIND),
        label: Arc::<str>::from("Reroute"),
        enabled: true,
        template: None,
        payload: serde_json::Value::Null,
    });
    menu_candidates.extend(candidates);

    let rows =
        super::super::searcher::build_rows(&menu_candidates, "", &canvas.interaction.recent_kinds);
    if rows.is_empty() {
        canvas.show_toast(
            host,
            window,
            DiagnosticSeverity::Info,
            "no insertable nodes for edge",
        );
        return;
    }

    let snapshot = canvas.sync_view_state(host);
    let bounds = canvas.interaction.last_bounds.unwrap_or_default();
    let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
    let origin = canvas.clamp_searcher_origin(invoked_at, visible, bounds, &snapshot);
    let active_row = rows
        .iter()
        .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
        .unwrap_or(0);

    canvas.interaction.context_menu = None;
    canvas.interaction.searcher = Some(SearcherState {
        origin,
        invoked_at,
        target: ContextMenuTarget::EdgeInsertNodePicker(edge),
        query: String::new(),
        candidates: menu_candidates,
        recent_kinds: canvas.interaction.recent_kinds.clone(),
        rows,
        hovered_row: None,
        active_row,
        scroll: 0,
    });
}

pub(super) fn open_edge_insert_context_menu<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut EventCx<'_, H>,
    edge: EdgeId,
    invoked_at: Point,
) {
    let candidates: Vec<InsertNodeCandidate> = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                presenter.list_insertable_nodes_for_edge(graph, edge)
            })
            .ok()
            .unwrap_or_default()
    };

    let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
    menu_candidates.push(InsertNodeCandidate {
        kind: NodeKindKey::new(REROUTE_KIND),
        label: Arc::<str>::from("Reroute"),
        enabled: true,
        template: None,
        payload: serde_json::Value::Null,
    });
    menu_candidates.extend(candidates);

    let mut items: Vec<NodeGraphContextMenuItem> = Vec::new();
    for (ix, c) in menu_candidates.iter().enumerate() {
        items.push(NodeGraphContextMenuItem {
            label: c.label.clone(),
            enabled: c.enabled,
            action: NodeGraphContextMenuAction::InsertNodeCandidate(ix),
        });
    }

    let snapshot = canvas.sync_view_state(cx.app);
    let origin = canvas.clamp_context_menu_origin(invoked_at, items.len(), cx.bounds, &snapshot);
    let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin,
        invoked_at,
        target: ContextMenuTarget::EdgeInsertNodePicker(edge),
        items,
        candidates: menu_candidates,
        hovered_item: None,
        active_item,
        typeahead: String::new(),
    });
}

pub(super) fn insert_node_on_edge<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut EventCx<'_, H>,
    edge: EdgeId,
    invoked_at: Point,
    candidate: InsertNodeCandidate,
) {
    enum Outcome {
        Apply(Vec<GraphOp>),
        Reject(DiagnosticSeverity, Arc<str>),
        Ignore,
    }

    canvas.record_recent_kind(&candidate.kind);

    let outcome = {
        let at = if candidate.kind.0 == REROUTE_KIND {
            canvas.reroute_pos_for_invoked_at(invoked_at)
        } else {
            CanvasPoint {
                x: invoked_at.x.0,
                y: invoked_at.y.0,
            }
        };

        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                let plan = presenter.plan_split_edge_candidate(graph, edge, &candidate, at);
                match plan.decision {
                    ConnectDecision::Accept => Outcome::Apply(plan.ops),
                    ConnectDecision::Reject => {
                        NodeGraphCanvas::toast_from_diagnostics(&plan.diagnostics)
                            .map(|(sev, msg)| Outcome::Reject(sev, msg))
                            .unwrap_or_else(|| {
                                Outcome::Reject(
                                    DiagnosticSeverity::Error,
                                    Arc::<str>::from(format!(
                                        "node insertion was rejected: {}",
                                        candidate.kind.0
                                    )),
                                )
                            })
                    }
                }
            })
            .ok()
            .unwrap_or(Outcome::Ignore)
    };

    match outcome {
        Outcome::Apply(ops) => {
            let select_node = candidate.kind.0 == REROUTE_KIND;
            let node_id = select_node
                .then(|| NodeGraphCanvas::first_added_node_id(&ops))
                .flatten();
            canvas.apply_ops(cx.app, cx.window, ops);
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
        Outcome::Reject(sev, msg) => canvas.show_toast(cx.app, cx.window, sev, msg),
        Outcome::Ignore => {}
    }
}
