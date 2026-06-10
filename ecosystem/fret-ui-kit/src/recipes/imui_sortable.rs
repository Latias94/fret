//! Immediate sortable/reorder helpers built on top of the typed `imui` drag/drop seam.
//!
//! This module intentionally lives in `recipes`, not in `imui` itself:
//! - `fret-ui-kit::imui` owns the typed drag/drop mechanism/helper boundary,
//! - this module owns reusable reorder packaging for immediate rows/lists/outliners,
//! - and app code still owns rendering plus the final domain mutation.

mod geometry;
mod reorder;
mod row;
mod types;

pub use geometry::vertical_insertion_side;
pub use reorder::reorder_vec_by_key;
pub use row::{sortable_row, sortable_row_with_options};
pub use types::{
    SortableInsertionSide, SortableRowOptions, SortableRowResponse, SortableRowSignal,
};
