mod header;
mod resize;

pub use header::TableHeaderResponse;
pub use resize::TableColumnResizeResponse;

/// Aggregated response surface for helper-owned table headers.
#[derive(Debug, Clone)]
pub struct TableResponse {
    pub(crate) headers: Vec<TableHeaderResponse>,
}

impl TableResponse {
    pub fn headers(&self) -> &[TableHeaderResponse] {
        &self.headers
    }

    pub fn header(&self, column_id: &str) -> Option<&TableHeaderResponse> {
        self.headers
            .iter()
            .find(|header| header.column_id.as_deref() == Some(column_id))
    }

    pub fn header_at(&self, column_index: usize) -> Option<&TableHeaderResponse> {
        self.headers
            .iter()
            .find(|header| header.column_index == column_index)
    }
}
