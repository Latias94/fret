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

    let snapshot = canvas.sync_view_state(host);
    let bounds = canvas.interaction.last_bounds.unwrap_or_default();
    let searcher = super::super::build_searcher_state(
        canvas,
        invoked_at,
        bounds,
        &snapshot,
        ContextMenuTarget::EdgeInsertNodePicker(edge),
        menu_candidates,
        canvas.interaction.recent_kinds.clone(),
        super::super::SearcherRowsMode::Catalog,
    );
    if searcher.rows.is_empty() {
        canvas.show_toast(
            host,
            window,
            DiagnosticSeverity::Info,
            "no insertable nodes for edge",
        );
        return;
    }

    canvas.interaction.context_menu = None;
    canvas.interaction.searcher = Some(searcher);
}
