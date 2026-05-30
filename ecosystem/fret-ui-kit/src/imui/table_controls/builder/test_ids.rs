use std::sync::Arc;

pub(super) fn row_test_id(
    explicit: Option<Arc<str>>,
    root: Option<&Arc<str>>,
    row_index: usize,
) -> Option<Arc<str>> {
    explicit.or_else(|| root.map(|base| Arc::from(format!("{base}.row.{row_index}"))))
}

pub(super) fn cell_test_id(row: Option<&Arc<str>>, cell_index: usize) -> Option<Arc<str>> {
    row.map(|base| Arc::from(format!("{base}.cell.{cell_index}")))
}
