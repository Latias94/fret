use std::sync::Arc;

use super::super::super::super::label_identity::parse_label_identity;
use super::TableColumn;

impl TableColumn {
    pub fn with_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn header(&self) -> Option<&str> {
        self.header.as_deref()
    }

    pub(crate) fn header_arc(&self) -> Option<Arc<str>> {
        self.header.clone()
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub(crate) fn id_arc(&self) -> Option<Arc<str>> {
        self.id.clone()
    }

    pub fn width(&self) -> super::TableColumnWidth {
        self.width
    }
}

pub(in crate::imui::options::collections::table_column) fn inferred_column_id(
    header: &str,
) -> Option<Arc<str>> {
    let identity = parse_label_identity(header).identity;
    (!identity.is_empty()).then(|| Arc::from(identity))
}
