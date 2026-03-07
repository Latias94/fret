use super::prelude::*;

pub(in super::super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    edge: EdgeId,
    invoked_at: Point,
) {
    let menu_candidates = canvas.list_edge_insert_candidates(host, edge);

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
