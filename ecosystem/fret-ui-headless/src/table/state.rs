use super::{
    ColumnFiltersState, ColumnOrderState, ColumnPinningState, ColumnSizingInfoState,
    ColumnSizingState, ColumnVisibilityState, ExpandingState, GlobalFilterState, GroupingState,
    PaginationState, RowPinningState, RowSelectionState, SortingState,
};

#[derive(Debug, Clone, Default)]
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
