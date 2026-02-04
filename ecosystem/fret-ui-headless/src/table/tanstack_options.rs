use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ColumnResizeDirection, ColumnResizeMode, TableOptions};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TanStackColumnResizeMode {
    OnChange,
    OnEnd,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TanStackColumnResizeDirection {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TanStackTableOptions {
    #[serde(default, rename = "manualFiltering")]
    pub manual_filtering: bool,
    #[serde(default = "default_true", rename = "enableFilters")]
    pub enable_filters: bool,
    #[serde(default = "default_true", rename = "enableColumnFilters")]
    pub enable_column_filters: bool,
    #[serde(default = "default_true", rename = "enableGlobalFilter")]
    pub enable_global_filter: bool,
    #[serde(default, rename = "manualSorting")]
    pub manual_sorting: bool,
    #[serde(default, rename = "manualPagination")]
    pub manual_pagination: bool,
    #[serde(default, rename = "manualExpanding")]
    pub manual_expanding: bool,
    #[serde(default, rename = "paginateExpandedRows")]
    pub paginate_expanded_rows: bool,
    #[serde(default, rename = "keepPinnedRows")]
    pub keep_pinned_rows: bool,
    #[serde(default = "default_true", rename = "enableSorting")]
    pub enable_sorting: bool,
    #[serde(default = "default_true", rename = "enableMultiSort")]
    pub enable_multi_sort: bool,
    #[serde(default, rename = "maxMultiSortColCount")]
    pub max_multi_sort_col_count: Option<usize>,
    #[serde(default = "default_true", rename = "enableSortingRemoval")]
    pub enable_sorting_removal: bool,
    #[serde(default = "default_true", rename = "enableMultiRemove")]
    pub enable_multi_remove: bool,
    #[serde(default, rename = "sortDescFirst")]
    pub sort_desc_first: Option<bool>,
    #[serde(default = "default_true", rename = "enableRowSelection")]
    pub enable_row_selection: bool,
    #[serde(default = "default_true", rename = "enableMultiRowSelection")]
    pub enable_multi_row_selection: bool,
    #[serde(default = "default_true", rename = "enableSubRowSelection")]
    pub enable_sub_row_selection: bool,
    #[serde(default = "default_true", rename = "enableColumnResizing")]
    pub enable_column_resizing: bool,
    #[serde(default = "default_true", rename = "enableHiding")]
    pub enable_hiding: bool,
    #[serde(default, rename = "columnResizeMode")]
    pub column_resize_mode: Option<TanStackColumnResizeMode>,
    #[serde(default, rename = "columnResizeDirection")]
    pub column_resize_direction: Option<TanStackColumnResizeDirection>,
}

impl Default for TanStackTableOptions {
    fn default() -> Self {
        Self {
            manual_filtering: false,
            enable_filters: true,
            enable_column_filters: true,
            enable_global_filter: true,
            manual_sorting: false,
            manual_pagination: false,
            manual_expanding: false,
            paginate_expanded_rows: true,
            keep_pinned_rows: true,
            enable_sorting: true,
            enable_multi_sort: true,
            max_multi_sort_col_count: None,
            enable_sorting_removal: true,
            enable_multi_remove: true,
            sort_desc_first: None,
            enable_row_selection: true,
            enable_multi_row_selection: true,
            enable_sub_row_selection: true,
            enable_column_resizing: true,
            enable_hiding: true,
            column_resize_mode: None,
            column_resize_direction: None,
        }
    }
}

impl TanStackTableOptions {
    pub fn from_json(value: &Value) -> serde_json::Result<Self> {
        serde_json::from_value(value.clone())
    }

    pub fn to_table_options(&self) -> TableOptions {
        let mut out = TableOptions::default();
        out.manual_filtering = self.manual_filtering;
        out.enable_filters = self.enable_filters;
        out.enable_column_filters = self.enable_column_filters;
        out.enable_global_filter = self.enable_global_filter;
        out.manual_sorting = self.manual_sorting;
        out.manual_pagination = self.manual_pagination;
        out.manual_expanding = self.manual_expanding;
        out.paginate_expanded_rows = self.paginate_expanded_rows;
        out.keep_pinned_rows = self.keep_pinned_rows;
        out.enable_sorting = self.enable_sorting;
        out.enable_multi_sort = self.enable_multi_sort;
        out.max_multi_sort_col_count = self.max_multi_sort_col_count;
        out.enable_sorting_removal = self.enable_sorting_removal;
        out.enable_multi_remove = self.enable_multi_remove;
        out.sort_desc_first = self.sort_desc_first;
        out.enable_row_selection = self.enable_row_selection;
        out.enable_multi_row_selection = self.enable_multi_row_selection;
        out.enable_sub_row_selection = self.enable_sub_row_selection;
        out.enable_column_resizing = self.enable_column_resizing;
        out.enable_hiding = self.enable_hiding;
        if let Some(mode) = self.column_resize_mode {
            out.column_resize_mode = match mode {
                TanStackColumnResizeMode::OnChange => ColumnResizeMode::OnChange,
                TanStackColumnResizeMode::OnEnd => ColumnResizeMode::OnEnd,
            };
        }
        if let Some(direction) = self.column_resize_direction {
            out.column_resize_direction = match direction {
                TanStackColumnResizeDirection::Ltr => ColumnResizeDirection::Ltr,
                TanStackColumnResizeDirection::Rtl => ColumnResizeDirection::Rtl,
            };
        }
        out
    }
}
