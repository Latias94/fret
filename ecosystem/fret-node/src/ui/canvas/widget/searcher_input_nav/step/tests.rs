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
