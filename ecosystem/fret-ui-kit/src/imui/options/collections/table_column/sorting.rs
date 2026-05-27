use super::{TableColumn, TableSortDirection};

impl TableColumn {
    pub fn is_sortable(&self) -> bool {
        self.sortable || self.sort_direction.is_some()
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn sort_direction(&self) -> Option<TableSortDirection> {
        self.sort_direction
    }

    pub fn sorted(mut self, direction: TableSortDirection) -> Self {
        self.sortable = true;
        self.sort_direction = Some(direction);
        self
    }

    pub fn with_sort_direction(mut self, direction: Option<TableSortDirection>) -> Self {
        self.sort_direction = direction;
        if direction.is_some() {
            self.sortable = true;
        }
        self
    }
}
