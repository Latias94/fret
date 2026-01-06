//! Headless table engine (TanStack-aligned vocabulary, Rust-native API).
//!
//! This module is feature-gated behind `fret-ui-kit/table`.

mod row_model;

pub use row_model::{Row, RowId, RowIndex, RowModel, Table, TableBuilder};
