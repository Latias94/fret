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
        let _ = apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(&mut searcher, -1.0);
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
