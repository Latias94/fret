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
mod tests {
    use super::*;

    fn header(label: &str) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Header,
            label: std::sync::Arc::<str>::from(label),
            enabled: false,
        }
    }

    fn candidate(label: &str, enabled: bool) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: 0 },
            label: std::sync::Arc::<str>::from(label),
            enabled,
        }
    }

    #[test]
    fn next_searcher_active_row_skips_headers_and_disabled_rows_forward() {
        let rows = vec![
            candidate("Disabled", false),
            header("Recent"),
            candidate("Enabled", true),
        ];

        assert_eq!(
            next_searcher_active_row(&rows, 0, SearcherStepDirection::Forward),
            Some(2)
        );
    }

    #[test]
    fn next_searcher_active_row_wraps_backwards() {
        let rows = vec![
            candidate("Enabled", true),
            header("Recent"),
            candidate("Other", true),
        ];

        assert_eq!(
            next_searcher_active_row(&rows, 0, SearcherStepDirection::Backward),
            Some(2)
        );
    }
}
