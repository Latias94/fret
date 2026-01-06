use super::{PaginationState, RowSelectionState, SortingState};

#[derive(Debug, Clone)]
pub struct TableState {
    pub sorting: SortingState,
    pub pagination: PaginationState,
    pub row_selection: RowSelectionState,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            sorting: SortingState::default(),
            pagination: PaginationState::default(),
            row_selection: RowSelectionState::default(),
        }
    }
}
