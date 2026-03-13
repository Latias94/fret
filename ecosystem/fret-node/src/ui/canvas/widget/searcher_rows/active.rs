use super::super::*;

pub(super) fn searcher_is_selectable_row(row: &SearcherRow) -> bool {
    matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
}

pub(super) fn searcher_first_selectable_row(rows: &[SearcherRow]) -> usize {
    rows.iter()
        .position(searcher_is_selectable_row)
        .unwrap_or(0)
}

pub(super) fn ensure_searcher_active_visible(searcher: &mut SearcherState) {
    let row_count = searcher.rows.len();
    if row_count == 0 {
        searcher.active_row = 0;
        searcher.scroll = 0;
        return;
    }

    let visible_rows = SEARCHER_MAX_VISIBLE_ROWS.min(row_count);
    let max_scroll = row_count.saturating_sub(visible_rows);
    searcher.scroll = searcher.scroll.min(max_scroll);

    if searcher.active_row < searcher.scroll {
        searcher.scroll = searcher.active_row;
    } else if searcher.active_row >= searcher.scroll + visible_rows {
        searcher.scroll = (searcher.active_row + 1).saturating_sub(visible_rows);
    }
    searcher.scroll = searcher.scroll.min(max_scroll);
}
