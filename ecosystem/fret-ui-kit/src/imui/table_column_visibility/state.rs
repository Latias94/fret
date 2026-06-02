use std::sync::Arc;

mod columns;
mod mutation;
mod overrides;
mod snapshot;
mod snapshot_io;

pub use snapshot::{TableColumnVisibilityEntry, TableColumnVisibilitySnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableColumnVisibilityOverride {
    id: Arc<str>,
    visible: bool,
}

/// Model state for runtime table-column visibility.
///
/// This intentionally stays policy-only: it maps stable column ids to visible flags and then
/// produces a new `TableColumn` list. Persistence, freeze panes, and durable column storage are
/// separate table policies.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImUiTableColumnVisibilityState {
    overrides: Vec<TableColumnVisibilityOverride>,
}
