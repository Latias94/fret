use std::sync::Arc;

use super::super::super::ResponseExt;

/// Response for one generated table-column visibility menu item.
#[derive(Debug, Clone)]
pub struct TableColumnVisibilityMenuItemResponse {
    column_id: Arc<str>,
    visible: bool,
    response: ResponseExt,
}

impl TableColumnVisibilityMenuItemResponse {
    pub(in crate::imui::table_column_visibility) fn new(
        column_id: Arc<str>,
        visible: bool,
        response: ResponseExt,
    ) -> Self {
        Self {
            column_id,
            visible,
            response,
        }
    }

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
