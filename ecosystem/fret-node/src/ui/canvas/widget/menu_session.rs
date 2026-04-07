mod context_menu;
mod searcher;

use super::*;

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn first_enabled_context_menu_item(items: &[NodeGraphContextMenuItem]) -> usize {
    context_menu::first_enabled_context_menu_item(items)
}

pub(super) fn build_searcher_rows(
    candidates: &[InsertNodeCandidate],
    query: &str,
    recent_kinds: &[NodeKindKey],
    rows_mode: SearcherRowsMode,
) -> Vec<SearcherRow> {
    searcher::build_searcher_rows(candidates, query, recent_kinds, rows_mode)
}

pub(super) fn build_context_menu_state<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    desired_origin: Point,
    bounds: Rect,
    snapshot: &ViewSnapshot,
    target: ContextMenuTarget,
    items: Vec<NodeGraphContextMenuItem>,
    candidates: Vec<InsertNodeCandidate>,
) -> ContextMenuState {
    context_menu::build_context_menu_state(
        canvas,
        desired_origin,
        bounds,
        snapshot,
        target,
        items,
        candidates,
    )
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
    searcher::build_searcher_state(
        canvas,
        desired_origin,
        bounds,
        snapshot,
        target,
        candidates,
        recent_kinds,
        rows_mode,
    )
}
