use std::sync::Arc;

use fret_ui::scroll::ScrollHandle;

#[derive(Clone)]
pub struct TableOptions {
    pub show_header: bool,
    pub striped: bool,
    pub clip_cells: bool,
    pub column_gap: crate::MetricRef,
    pub row_gap: crate::MetricRef,
    pub horizontal_scroll: Option<ScrollHandle>,
    pub test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for TableOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableOptions")
            .field("show_header", &self.show_header)
            .field("striped", &self.striped)
            .field("clip_cells", &self.clip_cells)
            .field("column_gap", &self.column_gap)
            .field("row_gap", &self.row_gap)
            .field("horizontal_scroll", &self.horizontal_scroll.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            striped: false,
            clip_cells: true,
            column_gap: crate::MetricRef::space(crate::Space::N0),
            row_gap: crate::MetricRef::space(crate::Space::N0),
            horizontal_scroll: None,
            test_id: None,
        }
    }
}
