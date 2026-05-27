use fret_core::Px;

use super::{TableColumn, TableColumnResizeOptions};

impl TableColumn {
    pub fn resizable(mut self) -> Self {
        self.resize = Some(TableColumnResizeOptions::default());
        self
    }

    pub fn resize_options(&self) -> Option<TableColumnResizeOptions> {
        self.resize
    }

    pub fn resizable_with_limits(mut self, min_width: Option<Px>, max_width: Option<Px>) -> Self {
        self.resize = Some(TableColumnResizeOptions {
            min_width,
            max_width,
        });
        self
    }
}
