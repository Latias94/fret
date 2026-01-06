//! Headless table engine (TanStack-aligned vocabulary, Rust-native API).
//!
//! This module is feature-gated behind `fret-ui-kit/table`.

mod column;
mod column_ordering;
mod column_pinning;
mod column_sizing;
mod column_sizing_info;
mod column_visibility;
mod faceting;
mod filtering;
mod flat_row_order;
mod memo;
mod options;
mod pagination;
mod row_expanding;
mod row_model;
mod row_pinning;
mod row_selection;
mod sorting;
mod state;

pub use column::{ColumnDef, ColumnHelper, ColumnId, FilterFn, SortCmpFn, create_column_helper};
pub use column_ordering::{ColumnOrderState, order_columns};
pub use column_pinning::{ColumnPinningState, split_pinned_columns};
pub use column_sizing::{ColumnSizingState, column_size};
pub use column_sizing_info::ColumnSizingInfoState;
pub use column_visibility::{ColumnVisibilityState, is_column_visible, visible_columns};
pub use faceting::{
    FacetCounts, FacetKey, FacetLabels, faceted_min_max_u64, faceted_row_model_excluding,
    faceted_unique_value_labels, faceted_unique_values,
};
pub use filtering::{
    ColumnFilter, ColumnFiltersState, GlobalFilterState, contains_ascii_case_insensitive,
    filter_row_model,
};
pub use flat_row_order::{FlatRowOrderCache, FlatRowOrderDeps, compute_flat_row_order};
pub use options::TableOptions;
pub use pagination::{PaginationState, paginate_row_model};
pub use row_expanding::{
    ExpandingState, expand_row_model, expanded_depth, is_row_expanded, is_some_rows_expanded,
    row_can_expand, row_is_all_parents_expanded, set_all_rows_expanded, toggle_all_rows_expanded,
    toggle_row_expanded,
};
pub use row_model::{Row, RowIndex, RowKey, RowModel, Table, TableBuilder};
pub use row_pinning::{
    RowPinPosition, RowPinningState, center_row_keys, is_row_pinned, is_some_rows_pinned, pin_row,
    pin_rows,
};
pub use row_selection::{
    RowSelectionState, SubRowSelection, is_all_rows_selected, is_row_selected,
    is_some_rows_selected, is_sub_row_selected, row_is_all_sub_rows_selected, row_is_some_selected,
    select_rows_fn, selected_flat_row_count, selected_root_row_count,
    toggle_all_page_rows_selected, toggle_all_rows_selected, toggle_row_selected,
};
pub use sorting::{
    SortSpec, SortingState, sort_for_column, sort_row_model, toggle_sort_for_column,
};
pub use state::TableState;
