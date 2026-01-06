//! Headless table engine (TanStack-aligned vocabulary, Rust-native API).
//!
//! This module is feature-gated behind `fret-ui-kit/table`.

mod column;
mod column_ordering;
mod column_pinning;
mod column_sizing;
mod column_sizing_info;
mod column_visibility;
mod pagination;
mod row_model;
mod row_selection;
mod sorting;
mod state;

pub use column::{ColumnDef, ColumnHelper, ColumnId, SortCmpFn, create_column_helper};
pub use column_ordering::{ColumnOrderState, order_columns};
pub use column_pinning::{ColumnPinningState, split_pinned_columns};
pub use column_sizing::{ColumnSizingState, column_size};
pub use column_sizing_info::ColumnSizingInfoState;
pub use column_visibility::{ColumnVisibilityState, is_column_visible, visible_columns};
pub use pagination::{PaginationState, paginate_row_model};
pub use row_model::{Row, RowIndex, RowKey, RowModel, Table, TableBuilder};
pub use row_selection::{RowSelectionState, is_row_selected, select_rows_fn};
pub use sorting::{SortSpec, SortingState, sort_row_model};
pub use state::TableState;
