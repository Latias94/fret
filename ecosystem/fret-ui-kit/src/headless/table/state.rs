use super::{
    ColumnOrderState, ColumnSizingState, ColumnVisibilityState, PaginationState, RowSelectionState,
    SortingState,
};

#[derive(Debug, Clone)]
pub struct TableState {
    pub sorting: SortingState,
    pub pagination: PaginationState,
    pub row_selection: RowSelectionState,
    pub column_visibility: ColumnVisibilityState,
    pub column_order: ColumnOrderState,
    pub column_sizing: ColumnSizingState,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            sorting: SortingState::default(),
            pagination: PaginationState::default(),
            row_selection: RowSelectionState::default(),
            column_visibility: ColumnVisibilityState::default(),
            column_order: ColumnOrderState::default(),
            column_sizing: ColumnSizingState::default(),
        }
    }
}
