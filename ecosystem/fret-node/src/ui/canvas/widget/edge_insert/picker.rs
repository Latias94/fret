use super::prelude::*;

pub(in super::super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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

    let rows = crate::ui::canvas::searcher::build_rows(
        &menu_candidates,
        "",
        &canvas.interaction.recent_kinds,
    );
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
