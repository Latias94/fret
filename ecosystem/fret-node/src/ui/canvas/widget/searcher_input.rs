use fret_core::{KeyCode, Modifiers};

use super::*;
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearcherStepDirection {
    Forward,
    Backward,
}

fn is_selectable_searcher_row(row: &SearcherRow) -> bool {
    matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
}

pub(super) fn next_searcher_active_row(
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

pub(super) fn apply_searcher_query_key(
    query: &mut String,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    match key {
        KeyCode::Backspace => {
            if query.is_empty() {
                return false;
            }
            query.pop();
            return true;
        }
        _ => {}
    }

    if modifiers.ctrl || modifiers.meta {
        return false;
    }

    let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) else {
        return false;
    };
    query.push(ch);
    true
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn try_activate_active_searcher_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
    ) -> bool {
        let Some(row_ix) = self
            .interaction
            .searcher
            .as_ref()
            .map(|searcher| searcher.active_row)
        else {
            return false;
        };
        self.try_activate_searcher_row(cx, row_ix)
    }

    pub(super) fn step_searcher_active_row(&mut self, direction: SearcherStepDirection) -> bool {
        let Some(searcher) = self.interaction.searcher.as_mut() else {
            return false;
        };
        let Some(next_ix) =
            next_searcher_active_row(&searcher.rows, searcher.active_row, direction)
        else {
            return false;
        };

        searcher.active_row = next_ix;
        Self::ensure_searcher_active_visible(searcher);
        true
    }

    pub(super) fn update_searcher_query_from_key(
        &mut self,
        key: KeyCode,
        modifiers: Modifiers,
    ) -> bool {
        let Some(searcher) = self.interaction.searcher.as_mut() else {
            return false;
        };
        if !apply_searcher_query_key(&mut searcher.query, key, modifiers) {
            return false;
        }

        Self::rebuild_searcher_rows(searcher);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn header(label: &str) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from(label),
            enabled: false,
        }
    }

    fn candidate(label: &str, enabled: bool) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: 0 },
            label: Arc::<str>::from(label),
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
        let rows = vec![candidate("A", true), header("Recent"), candidate("B", true)];

        assert_eq!(
            next_searcher_active_row(&rows, 0, SearcherStepDirection::Backward),
            Some(2)
        );
    }

    #[test]
    fn apply_searcher_query_key_handles_ascii_and_backspace() {
        let mut query = String::new();

        assert!(apply_searcher_query_key(
            &mut query,
            KeyCode::KeyA,
            Modifiers::default(),
        ));
        assert_eq!(query, "a");

        assert!(!apply_searcher_query_key(
            &mut query,
            KeyCode::KeyB,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        ));
        assert_eq!(query, "a");

        assert!(apply_searcher_query_key(
            &mut query,
            KeyCode::Backspace,
            Modifiers::default(),
        ));
        assert!(query.is_empty());
    }
}
