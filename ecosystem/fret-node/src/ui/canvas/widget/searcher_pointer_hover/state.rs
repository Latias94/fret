use super::super::*;
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};

fn is_selectable_searcher_row(row: &SearcherRow) -> bool {
    matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
}

pub(in super::super) fn sync_searcher_hovered_row<M: NodeGraphCanvasMiddleware>(
    searcher: &mut SearcherState,
    hovered_row: Option<usize>,
) -> bool {
    if searcher.hovered_row == hovered_row {
        return false;
    }

    searcher.hovered_row = hovered_row;
    if let Some(ix) = hovered_row
        && searcher
            .rows
            .get(ix)
            .is_some_and(is_selectable_searcher_row)
    {
        searcher.active_row = ix;
        NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    }
    true
}

#[cfg(test)]
mod tests;
