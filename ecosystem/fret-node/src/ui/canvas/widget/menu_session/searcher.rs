use super::super::*;

pub(super) fn build_searcher_rows(
    candidates: &[InsertNodeCandidate],
    query: &str,
    recent_kinds: &[NodeKindKey],
    rows_mode: SearcherRowsMode,
) -> Vec<SearcherRow> {
    match rows_mode {
        SearcherRowsMode::Catalog => {
            crate::ui::canvas::searcher::build_rows(candidates, query, recent_kinds)
        }
        SearcherRowsMode::Flat => crate::ui::canvas::searcher::build_rows_flat(candidates, query),
    }
}

pub(super) fn build_searcher_state<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    desired_origin: Point,
    bounds: Rect,
    snapshot: &ViewSnapshot,
    target: ContextMenuTarget,
    candidates: Vec<InsertNodeCandidate>,
    recent_kinds: Vec<NodeKindKey>,
    rows_mode: SearcherRowsMode,
) -> SearcherState {
    let rows = build_searcher_rows(&candidates, "", &recent_kinds, rows_mode);
    let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
    let origin = canvas.clamp_searcher_origin(desired_origin, visible, bounds, snapshot);
    let active_row = NodeGraphCanvasWith::<M>::searcher_first_selectable_row(&rows)
        .min(rows.len().saturating_sub(1));

    SearcherState {
        origin,
        invoked_at: desired_origin,
        target,
        rows_mode,
        query: String::new(),
        candidates,
        recent_kinds,
        rows,
        hovered_row: None,
        active_row,
        scroll: 0,
    }
}
