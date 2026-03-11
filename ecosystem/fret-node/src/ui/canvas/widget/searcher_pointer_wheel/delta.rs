use super::super::*;

pub(in super::super) fn apply_searcher_wheel_delta<M: NodeGraphCanvasMiddleware>(
    searcher: &mut SearcherState,
    delta_y: f32,
) -> bool {
    let n = searcher.rows.len();
    if n == 0 {
        return false;
    }

    let visible = super::super::SEARCHER_MAX_VISIBLE_ROWS.min(n);
    let max_scroll = n.saturating_sub(visible);
    let next_scroll = if delta_y > 0.0 {
        searcher.scroll.saturating_sub(1)
    } else if delta_y < 0.0 {
        (searcher.scroll + 1).min(max_scroll)
    } else {
        searcher.scroll
    };

    if next_scroll == searcher.scroll {
        return false;
    }

    searcher.scroll = next_scroll;
    NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CanvasPoint, NodeKindKey};
    use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
    use fret_core::{Point, Px};

    fn candidate(label: &str, enabled: bool, candidate_ix: usize) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix },
            label: std::sync::Arc::<str>::from(label),
            enabled,
        }
    }

    fn candidate_item(kind: &str, label: &str) -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new(kind),
            label: std::sync::Arc::<str>::from(label),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }
    }

    fn searcher_state(rows: Vec<SearcherRow>, active_row: usize, scroll: usize) -> SearcherState {
        SearcherState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(0.0), Px(0.0)),
            target: ContextMenuTarget::BackgroundInsertNodePicker {
                at: CanvasPoint::default(),
            },
            rows_mode: SearcherRowsMode::Catalog,
            query: String::new(),
            candidates: vec![candidate_item("math.add", "Math/Add")],
            recent_kinds: Vec::new(),
            rows,
            hovered_row: None,
            active_row,
            scroll,
        }
    }

    #[test]
    fn apply_searcher_wheel_delta_clamps_scroll_range() {
        let rows = (0..20)
            .map(|ix| candidate(&format!("Item {ix}"), true, 0))
            .collect();
        let mut searcher = searcher_state(rows, SEARCHER_MAX_VISIBLE_ROWS, 0);

        assert!(apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(
            &mut searcher,
            -1.0,
        ));
        assert_eq!(searcher.scroll, 1);

        for _ in 0..50 {
            let _ =
                apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(&mut searcher, -1.0);
        }
        assert_eq!(
            searcher.scroll,
            searcher
                .rows
                .len()
                .saturating_sub(SEARCHER_MAX_VISIBLE_ROWS)
        );

        for _ in 0..50 {
            let _ = apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(&mut searcher, 1.0);
        }
        assert_eq!(searcher.scroll, 1);
    }
}
