/// Headless table options (TanStack-aligned semantics, Rust-native API).
#[derive(Debug, Clone, Copy)]
pub struct TableOptions {
    /// If enabled, filtering is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `filtered_row_model()` returns `pre_filtered_row_model()`.
    pub manual_filtering: bool,
    /// If enabled, sorting is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `sorted_row_model()` returns `pre_sorted_row_model()`.
    pub manual_sorting: bool,
    /// If enabled, pagination is assumed to be done externally (e.g. server-side).
    ///
    /// When `true`, `row_model()` returns `pre_pagination_row_model()`.
    pub manual_pagination: bool,
    /// If enabled, expanded row handling is assumed to be done externally.
    ///
    /// When `true`, `expanded_row_model()` returns `pre_expanded_row_model()`.
    pub manual_expanding: bool,
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
    /// Whether to allow column pinning at the table level (TanStack `enablePinning`).
    pub enable_column_pinning: bool,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            manual_filtering: false,
            manual_sorting: false,
            manual_pagination: false,
            manual_expanding: false,
            paginate_expanded_rows: true,
            keep_pinned_rows: true,
            enable_hiding: true,
            enable_column_ordering: true,
            enable_column_pinning: true,
        }
    }
}
