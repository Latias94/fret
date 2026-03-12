use super::*;

fn searcher_with_rows(rows: Vec<SearcherRow>) -> SearcherState {
    SearcherState {
        origin: Point::default(),
        invoked_at: Point::default(),
        target: ContextMenuTarget::Background,
        rows_mode: SearcherRowsMode::Catalog,
        query: String::new(),
        candidates: Vec::new(),
        recent_kinds: Vec::new(),
        rows,
        hovered_row: None,
        active_row: 0,
        scroll: 0,
    }
}

#[test]
fn searcher_row_activation_item_returns_insert_action_for_enabled_candidate() {
    let searcher = searcher_with_rows(vec![SearcherRow {
        kind: SearcherRowKind::Candidate { candidate_ix: 3 },
        label: Arc::from("Add Node"),
        enabled: true,
    }]);

    let item = searcher_row_activation_item(&searcher, 0).expect("candidate item");
    assert_eq!(item.label.as_ref(), "Add Node");
    assert!(item.enabled);
    assert!(matches!(
        item.action,
        NodeGraphContextMenuAction::InsertNodeCandidate(3)
    ));
}

#[test]
fn searcher_row_activation_item_rejects_disabled_and_header_rows() {
    let disabled = searcher_with_rows(vec![SearcherRow {
        kind: SearcherRowKind::Candidate { candidate_ix: 1 },
        label: Arc::from("Disabled"),
        enabled: false,
    }]);
    assert!(searcher_row_activation_item(&disabled, 0).is_none());

    let header = searcher_with_rows(vec![SearcherRow {
        kind: SearcherRowKind::Header,
        label: Arc::from("Header"),
        enabled: true,
    }]);
    assert!(searcher_row_activation_item(&header, 0).is_none());
    assert!(searcher_row_activation_item(&header, 1).is_none());
}
