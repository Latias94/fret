//! Headless table engine (TanStack-aligned vocabulary, Rust-native API).
//!
//! This module is feature-gated behind `fret-ui-kit/table`.

mod column;
mod pagination;
mod row_model;
mod row_selection;
mod sorting;
mod state;

pub use column::{ColumnDef, ColumnHelper, ColumnId, SortCmpFn, create_column_helper};
pub use pagination::{PaginationState, paginate_row_model};
pub use row_model::{Row, RowId, RowIndex, RowModel, Table, TableBuilder};
pub use row_selection::{RowSelectionState, is_row_selected, select_rows_fn};
pub use sorting::{SortSpec, SortingState, sort_row_model};
pub use state::TableState;
