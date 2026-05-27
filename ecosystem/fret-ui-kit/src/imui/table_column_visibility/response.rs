use std::sync::Arc;

use super::super::ResponseExt;

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

/// Response for one generated table-column visibility menu item.
#[derive(Debug, Clone)]
pub struct TableColumnVisibilityMenuItemResponse {
    pub(super) column_id: Arc<str>,
    pub(super) visible: bool,
    pub(super) response: ResponseExt,
}

impl TableColumnVisibilityMenuResponse {
    pub fn items(&self) -> &[TableColumnVisibilityMenuItemResponse] {
        &self.items
    }

    pub fn item(&self, column_id: &str) -> Option<&TableColumnVisibilityMenuItemResponse> {
        self.items
            .iter()
            .find(|item| item.column_id.as_ref() == column_id)
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

impl TableColumnVisibilityMenuItemResponse {
    pub fn column_id(&self) -> &str {
        self.column_id.as_ref()
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn response(&self) -> ResponseExt {
        self.response
    }

    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn changed(&self) -> bool {
        self.response.changed()
    }
}
