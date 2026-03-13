use super::super::searcher_input::SearcherStepDirection;
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};

fn is_selectable_searcher_row(row: &SearcherRow) -> bool {
    matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
}

pub(in super::super) fn next_searcher_active_row(
    rows: &[SearcherRow],
    active_row: usize,
    direction: SearcherStepDirection,
) -> Option<usize> {
    let n = rows.len();
    if n == 0 {
        return None;
    }

    let mut ix = match direction {
        SearcherStepDirection::Forward => (active_row + 1) % n,
        SearcherStepDirection::Backward => {
            if active_row == 0 {
                n - 1
            } else {
                active_row - 1
            }
        }
    };

    for _ in 0..n {
        if rows.get(ix).is_some_and(is_selectable_searcher_row) {
            return Some(ix);
        }
        ix = match direction {
            SearcherStepDirection::Forward => (ix + 1) % n,
            SearcherStepDirection::Backward => {
                if ix == 0 {
                    n - 1
                } else {
                    ix - 1
                }
            }
        };
    }

    None
}

#[cfg(test)]
mod tests;
