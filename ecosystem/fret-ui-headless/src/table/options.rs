use super::{ColumnResizeDirection, ColumnResizeMode, GroupedColumnMode};

/// Headless table options (TanStack-aligned semantics, Rust-native API).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableOptions {
    /// Enables/disables pinning features globally (TanStack `enablePinning`).
    pub enable_pinning: bool,
    /// If enabled, filtering is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `filtered_row_model()` returns `pre_filtered_row_model()`.
    pub manual_filtering: bool,
    /// Enables/disables all filtering features (TanStack `enableFilters`).
    pub enable_filters: bool,
    /// Enables/disables per-column filtering (TanStack `enableColumnFilters`).
    pub enable_column_filters: bool,
    /// Enables/disables global filtering (TanStack `enableGlobalFilter`).
    pub enable_global_filter: bool,
    /// If enabled, sorting is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `sorted_row_model()` returns `pre_sorted_row_model()`.
    pub manual_sorting: bool,
    /// If enabled, pagination is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `row_model()` returns `pre_pagination_row_model()`.
    pub manual_pagination: bool,
    /// TanStack-aligned: pagination auto reset gate (`autoResetPageIndex`).
    pub auto_reset_page_index: Option<bool>,
    /// TanStack-aligned: total page count hint (`pageCount`).
    ///
    /// When set to `-1`, the page count is treated as unknown.
    pub page_count: Option<i32>,
    /// TanStack-aligned: total row count hint (`rowCount`).
    pub row_count: Option<usize>,
    /// If enabled, expanded row handling is assumed to be done externally.
    ///
    /// When `true`, `expanded_row_model()` returns `pre_expanded_row_model()`.
    pub manual_expanding: bool,
    /// Enables/disables expanding for all rows (TanStack `enableExpanding`).
    pub enable_expanding: bool,
    /// TanStack-aligned: global auto reset gate (`autoResetAll`).
    pub auto_reset_all: Option<bool>,
    /// TanStack-aligned: expanded auto reset gate (`autoResetExpanded`).
    pub auto_reset_expanded: Option<bool>,
    /// When true, pagination counts expanded rows (children) as part of the page.
    ///
    /// This mirrors TanStack's `paginateExpandedRows` behavior.
    pub paginate_expanded_rows: bool,
    /// If true, pinned rows can remain visible even if they are outside the current
    /// filtered/sorted/paginated row set (TanStack `keepPinnedRows`).
    pub keep_pinned_rows: bool,
    /// Whether to allow column hiding at the table level (TanStack `enableHiding`).
    pub enable_hiding: bool,
    /// Whether to allow column ordering at the table level (TanStack `enableColumnOrdering`).
    pub enable_column_ordering: bool,
    /// Whether to allow column pinning at the table level (TanStack `enableColumnPinning` with
    /// `enablePinning` fallback).
    pub enable_column_pinning: bool,
    /// Whether to allow row pinning at the table level (TanStack `enableRowPinning`).
    pub enable_row_pinning: bool,
    /// Whether to allow column resizing at the table level (TanStack `enableColumnResizing`).
    pub enable_column_resizing: bool,
    /// Enables/disables grouping for the table (TanStack `enableGrouping`).
    pub enable_grouping: bool,
    /// Enables/disables sorting for the table (TanStack `enableSorting`).
    pub enable_sorting: bool,
    /// Enables/disables multi-sort for the table (TanStack `enableMultiSort`).
    pub enable_multi_sort: bool,
    /// Maximum number of columns that can be multi-sorted (TanStack `maxMultiSortColCount`).
    pub max_multi_sort_col_count: Option<usize>,
    /// Enables/disables the ability to remove sorting (TanStack `enableSortingRemoval`).
    pub enable_sorting_removal: bool,
    /// Enables/disables the ability to remove multi-sorts (TanStack `enableMultiRemove`).
    pub enable_multi_remove: bool,
    /// When set, all sort toggles default to descending as their first toggle state
    /// (TanStack `sortDescFirst`).
    pub sort_desc_first: Option<bool>,
    /// Enables/disables row selection for the table (TanStack `enableRowSelection`).
    pub enable_row_selection: bool,
    /// Enables/disables multi-row selection for the table (TanStack `enableMultiRowSelection`).
    pub enable_multi_row_selection: bool,
    /// Enables/disables sub-row selection propagation for the table (TanStack `enableSubRowSelection`).
    pub enable_sub_row_selection: bool,
    /// If enabled, grouping is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `grouped_row_model()` returns `pre_grouped_row_model()`.
    pub manual_grouping: bool,
    /// Determines how grouped columns are ordered in the leaf column list (TanStack
    /// `groupedColumnMode`).
    pub grouped_column_mode: GroupedColumnMode,
    /// Determines when `column_sizing` updates during a resize interaction (TanStack
    /// `columnResizeMode`).
    pub column_resize_mode: ColumnResizeMode,
    /// Column resize direction for RTL layouts (TanStack `columnResizeDirection`).
    pub column_resize_direction: ColumnResizeDirection,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            enable_pinning: true,
            manual_filtering: false,
            enable_filters: true,
            enable_column_filters: true,
            enable_global_filter: true,
            manual_sorting: false,
            manual_pagination: false,
            auto_reset_page_index: None,
            page_count: None,
            row_count: None,
            manual_expanding: false,
            enable_expanding: true,
            auto_reset_all: None,
            auto_reset_expanded: None,
            paginate_expanded_rows: true,
            keep_pinned_rows: true,
            enable_hiding: true,
            enable_column_ordering: true,
            enable_column_pinning: true,
            enable_row_pinning: true,
            enable_column_resizing: true,
            enable_grouping: true,
            enable_sorting: true,
            enable_multi_sort: true,
            max_multi_sort_col_count: None,
            enable_sorting_removal: true,
            enable_multi_remove: true,
            sort_desc_first: None,
            enable_row_selection: true,
            enable_multi_row_selection: true,
            enable_sub_row_selection: true,
            manual_grouping: false,
            grouped_column_mode: GroupedColumnMode::Reorder,
            column_resize_mode: ColumnResizeMode::OnEnd,
            column_resize_direction: ColumnResizeDirection::Ltr,
        }
    }
}
