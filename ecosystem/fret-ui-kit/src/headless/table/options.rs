/// Headless table options (TanStack-aligned semantics, Rust-native API).
#[derive(Debug, Clone, Copy, Default)]
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
}
