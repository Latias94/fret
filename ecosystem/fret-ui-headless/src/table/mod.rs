//! Headless table engine (TanStack-aligned vocabulary, Rust-native API).
//!
//! This module is always available (no long-lived feature gate).

mod aggregation;
mod aggregation_fns;
mod cells;
mod column;
mod column_ordering;
mod column_pinning;
mod column_sizing;
mod column_sizing_info;
mod column_visibility;
mod core_model;
mod faceting;
mod filtering;
mod flat_row_order;
mod grouped_aggregation;
mod grouped_sorting;
mod grouping;
mod headers;
mod memo;
mod options;
mod pagination;
mod row_expanding;
mod row_model;
mod row_pinning;
mod row_selection;
mod sorting;
mod state;
mod tanstack_memo;
mod tanstack_options;
mod tanstack_state;
mod updater;

pub use aggregation::{Aggregation, aggregate_u64};
pub use aggregation_fns::{
    AggregationFn, AggregationFnSpec, BuiltInAggregationFn, apply_builtin_aggregation,
    resolve_auto_aggregation,
};
pub use cells::{CellSnapshot, RowCellsSnapshot, snapshot_cells_for_row};
pub use column::{
    BuiltInFilterFn, BuiltInSortingFn, ColumnDef, ColumnHelper, ColumnId, FilterFn,
    FilterFnWithMeta, FilteringFnSpec, SortCmpFn, SortUndefined, SortValueFn, SortingFnSpec,
    TanStackValue, ValueU64Fn, create_column_helper,
};
pub use column_ordering::{ColumnOrderState, order_columns};
pub use column_ordering::{move_column, moved_column, set_column_order, set_column_order_for};
pub use column_pinning::{
    ColumnPinPosition, ColumnPinningState, is_column_pinned, is_some_columns_pinned, pin_column,
    pin_columns, pinned_column, pinned_columns, split_pinned_columns,
};
pub use column_sizing::{
    ColumnResizeDirection, ColumnResizeMode, begin_column_resize, column_can_resize,
    column_resize_preview_size, drag_column_resize, end_column_resize, resolved_column_size,
};
pub use column_sizing::{ColumnSizingRegion, ColumnSizingState, column_size};
pub use column_sizing_info::ColumnSizingInfoState;
pub use column_visibility::{ColumnVisibilityState, is_column_visible, visible_columns};
pub use column_visibility::{set_column_visible, toggle_column_visible, toggled_column_visible};
pub use core_model::{
    ColumnCapabilitySnapshot, ColumnNodeSnapshot, CoreModelSnapshot, CoreRowsSnapshot,
    HeaderSizingSnapshot, LeafColumnsSnapshot, RowModelIdSnapshot,
};
pub use faceting::{
    FacetCounts, FacetKey, FacetLabels, faceted_min_max_u64, faceted_row_model_excluding,
    faceted_unique_value_labels, faceted_unique_values,
};
pub use filtering::{
    ColumnFilter, ColumnFiltersState, FilterFnDef, GlobalFilterState, RowColumnFilters,
    RowColumnFiltersMeta, RowFilterStateSnapshot, contains_ascii_case_insensitive,
    evaluate_row_filter_state, filter_row_model, set_column_filter_value_tanstack,
};
pub use flat_row_order::{FlatRowOrderCache, FlatRowOrderDeps, compute_flat_row_order};
pub use grouped_aggregation::{
    compute_grouped_u64_aggregations, compute_grouped_u64_aggregations_from_core,
};
pub use grouped_sorting::sort_grouped_row_indices_in_place;
pub use grouping::{
    GroupedColumnMode, GroupedRow, GroupedRowIndex, GroupedRowKind, GroupedRowModel, GroupingState,
    column_can_group, group_row_model, grouped_index, grouped_row_model_from_leaf,
    is_column_grouped, order_column_refs_for_grouping, order_columns_for_grouping, set_grouping,
    toggle_column_grouping, toggle_column_grouping_value, toggled_column_grouping,
    toggled_column_grouping_value,
};
pub use headers::{HeaderGroupSnapshot, HeaderSnapshot, build_header_groups};
pub use options::TableOptions;
pub use pagination::{PaginationBounds, pagination_bounds};
pub use pagination::{PaginationState, paginate_row_model};
pub use row_expanding::{
    ExpandingState, expand_row_model, expanded_depth, is_row_expanded, is_some_rows_expanded,
    row_can_expand, row_is_all_parents_expanded, set_all_rows_expanded, toggle_all_rows_expanded,
    toggle_row_expanded,
};
pub use row_model::{Row, RowId, RowIndex, RowKey, RowModel, Table, TableBuilder};
pub use row_pinning::{
    RowPinPosition, RowPinningState, center_row_keys, is_row_pinned, is_some_rows_pinned, pin_row,
    pin_row_keys, pin_rows,
};
pub use row_selection::{
    RowSelectionState, SubRowSelection, is_all_page_rows_selected, is_all_rows_selected,
    is_row_selected, is_some_page_rows_selected, is_some_rows_selected, is_sub_row_selected,
    row_is_all_sub_rows_selected, row_is_some_selected, select_rows_fn, selected_flat_row_count,
    selected_root_row_count, toggle_all_page_rows_selected, toggle_all_rows_selected,
    toggle_row_selected,
};
pub use sorting::{
    SortSpec, SortToggleColumn, SortingFnDef, SortingState, sort_for_column, sort_row_model,
    toggle_sort_for_column, toggle_sorting_handler_tanstack, toggle_sorting_state_handler_tanstack,
    toggle_sorting_state_tanstack, toggle_sorting_tanstack,
};
pub use state::TableState;
pub use tanstack_memo::{
    FlatRowOrderEntry, TanStackSortedFlatRowOrderCache, TanStackSortedFlatRowOrderDeps,
};
pub use tanstack_options::TanStackTableOptions;
pub use tanstack_state::{
    TanStackColumnFilter, TanStackPaginationState, TanStackSortingSpec, TanStackStateError,
    TanStackTableState,
};
pub use updater::{Updater, functional_update};
