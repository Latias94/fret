use super::*;

fn row(label: &str) -> SearcherRow {
    SearcherRow {
        kind: SearcherRowKind::Candidate { candidate_ix: 0 },
        label: std::sync::Arc::<str>::from(label),
        enabled: true,
    }
}

fn test_searcher(row_count: usize, scroll: usize) -> SearcherState {
    SearcherState {
        origin: Point::new(Px(10.0), Px(20.0)),
        invoked_at: Point::new(Px(10.0), Px(20.0)),
        target: ContextMenuTarget::Background,
        rows_mode: SearcherRowsMode::Flat,
        query: String::new(),
        candidates: Vec::new(),
        recent_kinds: Vec::new(),
        rows: (0..row_count).map(|ix| row(&format!("Row {ix}"))).collect(),
        hovered_row: None,
        active_row: 0,
        scroll,
    }
}

#[test]
fn searcher_visible_rows_caps_by_scroll_and_max_visible() {
    let searcher = test_searcher(20, 3);
    assert_eq!(searcher_visible_rows(&searcher), SEARCHER_MAX_VISIBLE_ROWS);
}

#[test]
fn hit_searcher_row_includes_scroll_offset() {
    let style = NodeGraphStyle::default();
    let searcher = test_searcher(5, 1);
    let hit = hit_searcher_row(&style, &searcher, Point::new(Px(15.0), Px(71.0)), 1.0);
    assert_eq!(hit, Some(1));
}
