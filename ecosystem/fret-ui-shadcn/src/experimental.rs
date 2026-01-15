//! Experimental/unstable component surfaces.
//!
//! This module intentionally groups prototypes that are useful for correctness validation and
//! interaction experiments, but are not recommended as default surfaces.
//!
//! Current contents:
//! - `DataGridElement`: element-based 2D grid prototype (rich per-cell UI; not spreadsheet-scale).

pub use crate::data_grid::{DataGrid as DataGridElement, DataGridRowState};
