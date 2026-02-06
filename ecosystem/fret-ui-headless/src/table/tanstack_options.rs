use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ColumnResizeDirection, ColumnResizeMode, GroupedColumnMode, TableOptions};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TanStackGroupedColumnModeStr {
    Reorder,
    Remove,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TanStackGroupedColumnMode {
    Bool(bool),
    Str(TanStackGroupedColumnModeStr),
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
    #[serde(default, rename = "filterFromLeafRows")]
    pub filter_from_leaf_rows: bool,
    #[serde(default, rename = "maxLeafRowFilterDepth")]
    pub max_leaf_row_filter_depth: Option<usize>,
    #[serde(default, rename = "manualSorting")]
    pub manual_sorting: bool,
    #[serde(default, rename = "manualPagination")]
    pub manual_pagination: bool,
    #[serde(default, rename = "autoResetPageIndex")]
    pub auto_reset_page_index: Option<bool>,
    #[serde(default, rename = "pageCount")]
    pub page_count: Option<i32>,
    #[serde(default, rename = "rowCount")]
    pub row_count: Option<usize>,
    #[serde(default, rename = "manualExpanding")]
    pub manual_expanding: bool,
    #[serde(default = "default_true", rename = "enableExpanding")]
    pub enable_expanding: bool,
    #[serde(default, rename = "autoResetAll")]
    pub auto_reset_all: Option<bool>,
    #[serde(default, rename = "autoResetExpanded")]
    pub auto_reset_expanded: Option<bool>,
    #[serde(default, rename = "manualGrouping")]
    pub manual_grouping: bool,
    #[serde(default = "default_true", rename = "paginateExpandedRows")]
    pub paginate_expanded_rows: bool,
    #[serde(default = "default_true", rename = "keepPinnedRows")]
    pub keep_pinned_rows: bool,
    #[serde(default, rename = "enableRowPinning")]
    pub enable_row_pinning: Option<bool>,
    #[serde(default = "default_true", rename = "enableGrouping")]
    pub enable_grouping: bool,
    #[serde(default, rename = "groupedColumnMode")]
    pub grouped_column_mode: Option<TanStackGroupedColumnMode>,
    #[serde(default, rename = "enableColumnPinning")]
    pub enable_column_pinning: Option<bool>,
    #[serde(default, rename = "enablePinning")]
    pub enable_pinning: Option<bool>,
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
            filter_from_leaf_rows: false,
            max_leaf_row_filter_depth: None,
            manual_sorting: false,
            manual_pagination: false,
            auto_reset_page_index: None,
            page_count: None,
            row_count: None,
            manual_expanding: false,
            enable_expanding: true,
            auto_reset_all: None,
            auto_reset_expanded: None,
            manual_grouping: false,
            paginate_expanded_rows: true,
            keep_pinned_rows: true,
            enable_row_pinning: None,
            enable_grouping: true,
            grouped_column_mode: None,
            enable_column_pinning: None,
            enable_pinning: None,
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
        out.enable_pinning = self.enable_pinning.unwrap_or(out.enable_pinning);
        out.manual_filtering = self.manual_filtering;
        out.enable_filters = self.enable_filters;
        out.enable_column_filters = self.enable_column_filters;
        out.enable_global_filter = self.enable_global_filter;
        out.filter_from_leaf_rows = self.filter_from_leaf_rows;
        out.max_leaf_row_filter_depth = self
            .max_leaf_row_filter_depth
            .unwrap_or(out.max_leaf_row_filter_depth);
        out.manual_sorting = self.manual_sorting;
        out.manual_pagination = self.manual_pagination;
        out.auto_reset_page_index = self.auto_reset_page_index;
        out.page_count = self.page_count;
        out.row_count = self.row_count;
        out.manual_expanding = self.manual_expanding;
        out.enable_expanding = self.enable_expanding;
        out.auto_reset_all = self.auto_reset_all;
        out.auto_reset_expanded = self.auto_reset_expanded;
        out.manual_grouping = self.manual_grouping;
        out.paginate_expanded_rows = self.paginate_expanded_rows;
        out.keep_pinned_rows = self.keep_pinned_rows;
        out.enable_row_pinning = self.enable_row_pinning.unwrap_or(out.enable_pinning);
        out.enable_grouping = self.enable_grouping;
        out.grouped_column_mode = match self.grouped_column_mode {
            None => out.grouped_column_mode,
            Some(TanStackGroupedColumnMode::Bool(false)) => GroupedColumnMode::None,
            Some(TanStackGroupedColumnMode::Bool(true)) => GroupedColumnMode::Reorder,
            Some(TanStackGroupedColumnMode::Str(TanStackGroupedColumnModeStr::Reorder)) => {
                GroupedColumnMode::Reorder
            }
            Some(TanStackGroupedColumnMode::Str(TanStackGroupedColumnModeStr::Remove)) => {
                GroupedColumnMode::Remove
            }
        };
        out.enable_column_pinning = self.enable_column_pinning.unwrap_or(out.enable_pinning);
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
