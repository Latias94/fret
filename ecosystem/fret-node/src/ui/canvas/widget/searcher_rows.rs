mod active;
mod recent;

use super::*;

const MAX_RECENT_SEARCHER_KINDS: usize = 20;

pub(super) fn record_recent_kind<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    kind: &NodeKindKey,
) {
    recent::record_recent_kind::<M>(canvas, kind, MAX_RECENT_SEARCHER_KINDS)
}

pub(super) fn searcher_is_selectable_row(row: &SearcherRow) -> bool {
    active::searcher_is_selectable_row(row)
}

pub(super) fn searcher_first_selectable_row(rows: &[SearcherRow]) -> usize {
    active::searcher_first_selectable_row(rows)
}

pub(super) fn rebuild_searcher_rows<M: NodeGraphCanvasMiddleware>(searcher: &mut SearcherState) {
    searcher.rows = build_searcher_rows(
        &searcher.candidates,
        &searcher.query,
        &searcher.recent_kinds,
        searcher.rows_mode,
    );
    searcher.scroll = searcher.scroll.min(
        searcher
            .rows
            .len()
            .saturating_sub(SEARCHER_MAX_VISIBLE_ROWS),
    );
    searcher.active_row = active::searcher_first_selectable_row(&searcher.rows)
        .min(searcher.rows.len().saturating_sub(1));
    active::ensure_searcher_active_visible(searcher);
}

pub(super) fn ensure_searcher_active_visible(searcher: &mut SearcherState) {
    active::ensure_searcher_active_visible(searcher)
}
