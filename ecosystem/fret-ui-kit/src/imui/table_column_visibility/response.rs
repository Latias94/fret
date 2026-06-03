mod item;

pub use item::TableColumnVisibilityMenuItemResponse;

/// Aggregated response for a helper-composed table-column visibility menu section.
#[derive(Debug, Clone, Default)]
pub struct TableColumnVisibilityMenuResponse {
    pub(super) items: Vec<TableColumnVisibilityMenuItemResponse>,
}

/// Response for helper-composed table header context-menu visibility policy.
#[derive(Debug, Clone, Default)]
pub struct TableColumnVisibilityHeaderContextMenuResponse {
    pub(super) open: bool,
    pub(super) items: TableColumnVisibilityMenuResponse,
}

impl TableColumnVisibilityMenuResponse {
    pub fn items(&self) -> &[TableColumnVisibilityMenuItemResponse] {
        &self.items
    }

    pub fn item(&self, column_id: &str) -> Option<&TableColumnVisibilityMenuItemResponse> {
        self.items.iter().find(|item| item.column_id() == column_id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn changed(&self) -> bool {
        self.items.iter().any(|item| item.changed())
    }
}

impl TableColumnVisibilityHeaderContextMenuResponse {
    pub fn open(&self) -> bool {
        self.open
    }

    pub fn items(&self) -> &TableColumnVisibilityMenuResponse {
        &self.items
    }

    pub fn changed(&self) -> bool {
        self.items.changed()
    }
}
