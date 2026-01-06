//! Headless table engine (TanStack-aligned vocabulary, Rust-native API).
//!
//! This module is feature-gated behind `fret-ui-kit/table`.

mod row_model;
mod row_selection;

pub use row_model::{Row, RowId, RowIndex, RowModel, Table, TableBuilder};
pub use row_selection::{RowSelectionState, is_row_selected, select_rows_fn};
