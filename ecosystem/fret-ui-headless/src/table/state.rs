use super::{
    ColumnFiltersState, ColumnOrderState, ColumnPinningState, ColumnSizingInfoState,
    ColumnSizingState, ColumnVisibilityState, ExpandingState, GlobalFilterState, GroupingState,
    PaginationState, RowPinningState, RowSelectionState, SortingState,
};

#[derive(Debug, Clone)]
pub struct TableState {
    pub sorting: SortingState,
    pub grouping: GroupingState,
    pub column_filters: ColumnFiltersState,
    pub global_filter: GlobalFilterState,
    pub pagination: PaginationState,
    pub expanding: ExpandingState,
    pub row_pinning: RowPinningState,
    pub row_selection: RowSelectionState,
    pub column_visibility: ColumnVisibilityState,
    pub column_order: ColumnOrderState,
    pub column_sizing: ColumnSizingState,
    pub column_sizing_info: ColumnSizingInfoState,
    pub column_pinning: ColumnPinningState,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            sorting: SortingState::default(),
            grouping: GroupingState::default(),
            column_filters: ColumnFiltersState::default(),
            global_filter: GlobalFilterState::default(),
            pagination: PaginationState::default(),
            expanding: ExpandingState::default(),
            row_pinning: RowPinningState::default(),
            row_selection: RowSelectionState::default(),
            column_visibility: ColumnVisibilityState::default(),
            column_order: ColumnOrderState::default(),
            column_sizing: ColumnSizingState::default(),
            column_sizing_info: ColumnSizingInfoState::default(),
            column_pinning: ColumnPinningState::default(),
        }
    }
}
